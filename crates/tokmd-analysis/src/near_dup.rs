//! Near-duplicate detection via Winnowing fingerprinting.
//!
//! Implements a content-based near-duplicate detection algorithm:
//! 1. Tokenize source text by splitting on non-alphanumeric boundaries
//! 2. Build k-grams (k=25 tokens) and hash each with FxHash
//! 3. Apply Winnowing (window size w=4) to select representative fingerprints
//! 4. Build inverted index from fingerprints to files
//! 5. Compute Jaccard similarity for candidate pairs
//! 6. Emit pairs exceeding the similarity threshold

use std::collections::{BTreeMap, HashMap};
use std::io::Read;
use std::path::Path;

use anyhow::Result;
use rustc_hash::FxHasher;
use std::hash::{Hash, Hasher};

use tokmd_analysis_types::{NearDupPairRow, NearDupParams, NearDupScope, NearDuplicateReport};
use tokmd_types::{ExportData, FileKind};

use crate::AnalysisLimits;

/// Default k-gram size (number of tokens per shingle).
const K: usize = 25;
/// Winnowing window size.
const W: usize = 4;
/// Skip fingerprints appearing in more than this many files (common boilerplate).
const MAX_POSTINGS: usize = 50;

/// Build a near-duplicate report for the given export data.
pub(crate) fn build_near_dup_report(
    root: &Path,
    export: &ExportData,
    scope: NearDupScope,
    threshold: f64,
    max_files: usize,
    limits: &AnalysisLimits,
) -> Result<NearDuplicateReport> {
    let max_file_bytes = limits.max_file_bytes.unwrap_or(512_000);
    let params = NearDupParams {
        scope,
        threshold,
        max_files,
    };

    // Collect eligible parent files
    let mut files: Vec<&tokmd_types::FileRow> = export
        .rows
        .iter()
        .filter(|r| r.kind == FileKind::Parent)
        .filter(|r| (r.bytes as u64) <= max_file_bytes)
        .collect();

    // Sort by code lines desc for determinism
    files.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.path.cmp(&b.path)));

    let files_skipped = if files.len() > max_files {
        let skipped = files.len() - max_files;
        files.truncate(max_files);
        skipped
    } else {
        0
    };

    let files_analyzed = files.len();

    // Partition files by scope
    let partitions = partition_files(&files, scope);

    let mut all_pairs: Vec<NearDupPairRow> = Vec::new();

    for partition in partitions {
        if partition.len() < 2 {
            continue;
        }

        // Compute fingerprints for each file in this partition
        let mut file_fingerprints: Vec<(usize, Vec<u64>)> = Vec::new();
        for &file_idx in &partition {
            let row = files[file_idx];
            let file_path = root.join(&row.path);
            match read_and_fingerprint(&file_path) {
                Ok(fps) if !fps.is_empty() => {
                    file_fingerprints.push((file_idx, fps));
                }
                _ => {}
            }
        }

        // Build inverted index: fingerprint -> list of (local_idx) into file_fingerprints
        let mut inverted: HashMap<u64, Vec<usize>> = HashMap::new();
        for (local_idx, (_, fps)) in file_fingerprints.iter().enumerate() {
            for &fp in fps {
                inverted.entry(fp).or_default().push(local_idx);
            }
        }

        // Count shared fingerprints per pair
        let mut pair_shared: BTreeMap<(usize, usize), usize> = BTreeMap::new();
        for posting_list in inverted.values() {
            if posting_list.len() > MAX_POSTINGS {
                continue;
            }
            for i in 0..posting_list.len() {
                for j in (i + 1)..posting_list.len() {
                    let a = posting_list[i];
                    let b = posting_list[j];
                    let key = if a <= b { (a, b) } else { (b, a) };
                    *pair_shared.entry(key).or_insert(0) += 1;
                }
            }
        }

        // Compute Jaccard similarity per pair
        for ((a, b), shared) in pair_shared {
            let fp_a = file_fingerprints[a].1.len();
            let fp_b = file_fingerprints[b].1.len();
            let union = fp_a + fp_b - shared;
            if union == 0 {
                continue;
            }
            let similarity = shared as f64 / union as f64;
            if similarity >= threshold {
                let idx_a = file_fingerprints[a].0;
                let idx_b = file_fingerprints[b].0;
                all_pairs.push(NearDupPairRow {
                    left: files[idx_a].path.clone(),
                    right: files[idx_b].path.clone(),
                    similarity: round4(similarity),
                    shared_fingerprints: shared,
                    left_fingerprints: fp_a,
                    right_fingerprints: fp_b,
                });
            }
        }
    }

    // Sort by similarity desc, then by left path
    all_pairs.sort_by(|a, b| {
        b.similarity
            .partial_cmp(&a.similarity)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.left.cmp(&b.left))
    });

    Ok(NearDuplicateReport {
        params,
        pairs: all_pairs,
        files_analyzed,
        files_skipped,
    })
}

/// Partition file indices by the specified scope.
fn partition_files(files: &[&tokmd_types::FileRow], scope: NearDupScope) -> Vec<Vec<usize>> {
    match scope {
        NearDupScope::Global => {
            vec![(0..files.len()).collect()]
        }
        NearDupScope::Module => {
            let mut map: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
            for (i, row) in files.iter().enumerate() {
                map.entry(&row.module).or_default().push(i);
            }
            map.into_values().collect()
        }
        NearDupScope::Lang => {
            let mut map: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
            for (i, row) in files.iter().enumerate() {
                map.entry(&row.lang).or_default().push(i);
            }
            map.into_values().collect()
        }
    }
}

/// Read a file and compute its Winnowing fingerprints.
fn read_and_fingerprint(path: &Path) -> Result<Vec<u64>> {
    let mut content = String::new();
    let mut file = std::fs::File::open(path)?;
    file.read_to_string(&mut content)?;

    Ok(winnow(&content))
}

/// Tokenize text by splitting on non-alphanumeric/underscore boundaries.
fn tokenize(text: &str) -> Vec<&str> {
    let mut tokens = Vec::new();
    let bytes = text.as_bytes();
    let mut start = None;

    for (i, &b) in bytes.iter().enumerate() {
        let is_token_char = b.is_ascii_alphanumeric() || b == b'_';
        match (start, is_token_char) {
            (None, true) => start = Some(i),
            (Some(s), false) => {
                tokens.push(&text[s..i]);
                start = None;
            }
            _ => {}
        }
    }
    if let Some(s) = start {
        tokens.push(&text[s..]);
    }
    tokens
}

/// Hash a k-gram (slice of tokens) using FxHash.
fn hash_kgram(tokens: &[&str]) -> u64 {
    let mut hasher = FxHasher::default();
    for t in tokens {
        t.hash(&mut hasher);
    }
    hasher.finish()
}

/// Apply the Winnowing algorithm to extract fingerprints from text.
fn winnow(text: &str) -> Vec<u64> {
    let tokens = tokenize(text);
    if tokens.len() < K {
        return Vec::new();
    }

    // Build k-gram hashes
    let kgram_count = tokens.len() - K + 1;
    let hashes: Vec<u64> = (0..kgram_count)
        .map(|i| hash_kgram(&tokens[i..i + K]))
        .collect();

    if hashes.len() < W {
        // Not enough hashes for winnowing; return all
        return hashes;
    }

    // Winnowing: in each window of W hashes, select the minimum
    let mut fingerprints = Vec::new();
    let mut prev_min_idx: Option<usize> = None;

    for window_start in 0..=(hashes.len() - W) {
        let window = &hashes[window_start..window_start + W];
        // Find rightmost minimum in window
        let mut min_val = window[0];
        let mut min_idx = window_start;
        for (offset, &h) in window.iter().enumerate() {
            if h <= min_val {
                min_val = h;
                min_idx = window_start + offset;
            }
        }

        if prev_min_idx != Some(min_idx) {
            fingerprints.push(min_val);
            prev_min_idx = Some(min_idx);
        }
    }

    fingerprints
}

fn round4(v: f64) -> f64 {
    (v * 10000.0).round() / 10000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_basic() {
        let tokens = tokenize("fn hello_world() { let x = 42; }");
        assert_eq!(tokens, vec!["fn", "hello_world", "let", "x", "42"]);
    }

    #[test]
    fn winnow_short_text_returns_empty() {
        assert!(winnow("short").is_empty());
    }

    #[test]
    fn winnow_produces_fingerprints() {
        let text = (0..100)
            .map(|i| format!("token{}", i))
            .collect::<Vec<_>>()
            .join(" ");
        let fps = winnow(&text);
        assert!(!fps.is_empty());
    }

    #[test]
    fn identical_texts_have_same_fingerprints() {
        let text = (0..100)
            .map(|i| format!("word{}", i % 20))
            .collect::<Vec<_>>()
            .join(" ");
        let fps1 = winnow(&text);
        let fps2 = winnow(&text);
        assert_eq!(fps1, fps2);
    }

    #[test]
    fn jaccard_of_identical_is_one() {
        let fps = [1u64, 2, 3, 4, 5];
        let shared = fps.len();
        let union = fps.len() + fps.len() - shared;
        let jaccard = shared as f64 / union as f64;
        assert!((jaccard - 1.0).abs() < 1e-10);
    }
}
