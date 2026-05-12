//! Near-duplicate detection via Winnowing fingerprinting.
//!
//! Implements a content-based near-duplicate detection algorithm:
//! 1. Tokenize source text by splitting on non-alphanumeric boundaries
//! 2. Build k-grams (k=25 tokens) and hash each with FxHash
//! 3. Apply Winnowing (window size w=4) to select representative fingerprints
//! 4. Build inverted index from fingerprints to files
//! 5. Compute Jaccard similarity for candidate pairs
//! 6. Emit pairs exceeding the similarity threshold

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::Result;
use globset::{Glob, GlobSetBuilder};

use tokmd_analysis_types::{
    NearDupAlgorithm, NearDupPairRow, NearDupParams, NearDupScope, NearDupStats,
    NearDuplicateReport,
};
use tokmd_types::{ExportData, FileKind};

mod clusters;
mod fingerprint;
use clusters::build_clusters;
use fingerprint::{K, MAX_POSTINGS, W, read_and_fingerprint};

#[cfg(test)]
use fingerprint::{tokenize, winnow};

#[cfg(test)]
#[path = "tests.rs"]
mod moved_tests;

/// Limits controlling file scope for near-duplicate fingerprinting.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct NearDupLimits {
    #[allow(dead_code)]
    pub(crate) max_bytes: Option<u64>,
    pub(crate) max_file_bytes: Option<u64>,
}

/// Build a near-duplicate report for the given export data.
#[allow(clippy::too_many_arguments)]
pub(crate) fn build_near_dup_report(
    root: &Path,
    export: &ExportData,
    scope: NearDupScope,
    threshold: f64,
    max_files: usize,
    max_pairs: Option<usize>,
    limits: &NearDupLimits,
    exclude_patterns: &[String],
) -> Result<NearDuplicateReport> {
    let max_file_bytes = limits.max_file_bytes.unwrap_or(512_000);
    let params = NearDupParams {
        scope,
        threshold,
        max_files,
        max_pairs,
        max_file_bytes: Some(max_file_bytes),
        selection_method: Some("top_by_code_lines_then_path".to_string()),
        algorithm: Some(NearDupAlgorithm {
            k_gram_size: K,
            window_size: W,
            max_postings: MAX_POSTINGS,
        }),
        exclude_patterns: exclude_patterns.to_vec(),
    };

    // Build glob set for exclusion patterns
    let glob_set = if exclude_patterns.is_empty() {
        None
    } else {
        let mut builder = GlobSetBuilder::new();
        for pattern in exclude_patterns {
            builder.add(Glob::new(pattern)?);
        }
        Some(builder.build()?)
    };

    // Collect eligible parent files
    let mut files: Vec<&tokmd_types::FileRow> = export
        .rows
        .iter()
        .filter(|r| r.kind == FileKind::Parent)
        .filter(|r| (r.bytes as u64) <= max_file_bytes)
        .collect();

    // Apply glob exclusion patterns
    let excluded_by_pattern = if let Some(ref gs) = glob_set {
        let before = files.len();
        files.retain(|r| !gs.is_match(&r.path));
        before - files.len()
    } else {
        0
    };

    // Sort by code lines desc for determinism
    files.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.path.cmp(&b.path)));

    let eligible_files = files.len();

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
    let mut bytes_processed: u64 = 0;

    let fp_start = std::time::Instant::now();

    // Phase 1: Fingerprinting
    // We collect all partition fingerprints first, then pair them.
    let mut partition_fps: Vec<Vec<(usize, Vec<u64>)>> = Vec::new();
    for partition in &partitions {
        let mut file_fingerprints: Vec<(usize, Vec<u64>)> = Vec::new();
        for &file_idx in partition {
            let row = files[file_idx];
            let file_path = root.join(&row.path);
            match read_and_fingerprint(&file_path) {
                Ok(mut fps) if !fps.is_empty() => {
                    fps.sort_unstable();
                    fps.dedup();
                    bytes_processed += row.bytes as u64;
                    file_fingerprints.push((file_idx, fps));
                }
                _ => {}
            }
        }
        partition_fps.push(file_fingerprints);
    }

    let fingerprinting_ms = fp_start.elapsed().as_millis() as u64;
    let pair_start = std::time::Instant::now();

    // Phase 2: Pairing
    for file_fingerprints in &partition_fps {
        if file_fingerprints.len() < 2 {
            continue;
        }

        // Build inverted index: fingerprint -> list of (local_idx) into file_fingerprints
        let mut inverted: BTreeMap<u64, Vec<usize>> = BTreeMap::new();
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
                    if a == b {
                        continue; // skip self-pairs
                    }
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

    let pairing_ms = pair_start.elapsed().as_millis() as u64;

    // Sort by similarity desc, then by left path, then by right path
    all_pairs.sort_by(|a, b| {
        b.similarity
            .partial_cmp(&a.similarity)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.left.cmp(&b.left))
            .then_with(|| a.right.cmp(&b.right))
    });

    // Build clusters from ALL pairs (before truncation)
    let clusters = if all_pairs.is_empty() {
        None
    } else {
        Some(build_clusters(&all_pairs))
    };

    // Then truncate pairs list
    let truncated = if let Some(cap) = max_pairs {
        if all_pairs.len() > cap {
            all_pairs.truncate(cap);
            true
        } else {
            false
        }
    } else {
        false
    };

    let stats = Some(NearDupStats {
        fingerprinting_ms,
        pairing_ms,
        bytes_processed,
    });

    Ok(NearDuplicateReport {
        params,
        pairs: all_pairs,
        files_analyzed,
        files_skipped,
        eligible_files: Some(eligible_files),
        clusters,
        truncated,
        excluded_by_pattern: if excluded_by_pattern > 0 {
            Some(excluded_by_pattern)
        } else {
            None
        },
        stats,
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

    // ---------------------------------------------------------------
    // Algorithm constants in params
    // ---------------------------------------------------------------

    #[test]
    fn pair_sort_deterministic_with_right_tiebreak() {
        let mut pairs = [
            NearDupPairRow {
                left: "a.rs".to_string(),
                right: "c.rs".to_string(),
                similarity: 0.90,
                shared_fingerprints: 10,
                left_fingerprints: 20,
                right_fingerprints: 20,
            },
            NearDupPairRow {
                left: "a.rs".to_string(),
                right: "b.rs".to_string(),
                similarity: 0.90,
                shared_fingerprints: 10,
                left_fingerprints: 20,
                right_fingerprints: 20,
            },
        ];
        pairs.sort_by(|a, b| {
            b.similarity
                .partial_cmp(&a.similarity)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.left.cmp(&b.left))
                .then_with(|| a.right.cmp(&b.right))
        });
        // Same similarity, same left => ordered by right
        assert_eq!(pairs[0].right, "b.rs");
        assert_eq!(pairs[1].right, "c.rs");
    }

    #[test]
    fn self_pair_guard_skips_same_index() {
        // If a posting list has the same local_idx twice (shouldn't happen
        // with deduped fingerprints, but belt-and-suspenders), the guard skips it.
        let posting_list = [0usize, 0, 1];
        let mut pair_count = 0;
        for i in 0..posting_list.len() {
            for j in (i + 1)..posting_list.len() {
                let a = posting_list[i];
                let b = posting_list[j];
                if a == b {
                    continue;
                }
                pair_count += 1;
            }
        }
        // (0,0) at (0,1) => skipped; (0,1) at (0,2) => counted; (0,1) at (1,2) => counted
        assert_eq!(pair_count, 2);
    }

    #[test]
    fn algorithm_constants_match() {
        let algo = NearDupAlgorithm {
            k_gram_size: K,
            window_size: W,
            max_postings: MAX_POSTINGS,
        };
        assert_eq!(algo.k_gram_size, 25);
        assert_eq!(algo.window_size, 4);
        assert_eq!(algo.max_postings, 50);
    }
}
