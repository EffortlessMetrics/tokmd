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
use std::io::Read;
use std::path::Path;

use anyhow::Result;
use globset::{Glob, GlobSetBuilder};
use rustc_hash::FxHasher;
use std::hash::{Hash, Hasher};

use tokmd_analysis_types::{
    NearDupAlgorithm, NearDupCluster, NearDupPairRow, NearDupParams, NearDupScope, NearDupStats,
    NearDuplicateReport,
};
use tokmd_types::{ExportData, FileKind};

/// Limits controlling file scope for near-duplicate fingerprinting.
#[derive(Debug, Clone, Copy, Default)]
pub struct NearDupLimits {
    pub max_bytes: Option<u64>,
    pub max_file_bytes: Option<u64>,
}

/// Default k-gram size (number of tokens per shingle).
const K: usize = 25;
/// Winnowing window size.
const W: usize = 4;
/// Skip fingerprints appearing in more than this many files (common boilerplate).
const MAX_POSTINGS: usize = 50;

/// Build a near-duplicate report for the given export data.
#[allow(clippy::too_many_arguments)]
pub fn build_near_dup_report(
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

// ---------------------------------------------------------------------------
// Union-Find (Disjoint Sets) for clustering
// ---------------------------------------------------------------------------

/// Path-compressed union-find with union by rank.
struct DisjointSets {
    parent: Vec<usize>,
    rank: Vec<usize>,
}

impl DisjointSets {
    fn new(n: usize) -> Self {
        Self {
            parent: (0..n).collect(),
            rank: vec![0; n],
        }
    }

    fn find(&mut self, x: usize) -> usize {
        if self.parent[x] != x {
            self.parent[x] = self.find(self.parent[x]);
        }
        self.parent[x]
    }

    fn union(&mut self, a: usize, b: usize) {
        let ra = self.find(a);
        let rb = self.find(b);
        if ra == rb {
            return;
        }
        match self.rank[ra].cmp(&self.rank[rb]) {
            std::cmp::Ordering::Less => self.parent[ra] = rb,
            std::cmp::Ordering::Greater => self.parent[rb] = ra,
            std::cmp::Ordering::Equal => {
                self.parent[rb] = ra;
                self.rank[ra] += 1;
            }
        }
    }
}

/// Build connected-component clusters from near-duplicate pairs.
fn build_clusters(pairs: &[NearDupPairRow]) -> Vec<NearDupCluster> {
    // Collect unique file names and assign indices
    let mut name_to_idx: BTreeMap<&str, usize> = BTreeMap::new();
    let mut names: Vec<&str> = Vec::new();
    for pair in pairs {
        for name in [pair.left.as_str(), pair.right.as_str()] {
            if !name_to_idx.contains_key(name) {
                let idx = names.len();
                name_to_idx.insert(name, idx);
                names.push(name);
            }
        }
    }

    let mut ds = DisjointSets::new(names.len());

    // Track per-file connection counts for representative selection
    let mut connection_count: BTreeMap<usize, usize> = BTreeMap::new();

    for pair in pairs {
        let a = name_to_idx[pair.left.as_str()];
        let b = name_to_idx[pair.right.as_str()];
        ds.union(a, b);
        *connection_count.entry(a).or_insert(0) += 1;
        *connection_count.entry(b).or_insert(0) += 1;
    }

    // Group files by root
    let mut components: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for i in 0..names.len() {
        let root = ds.find(i);
        components.entry(root).or_default().push(i);
    }

    // Track max similarity and pair count per component
    let mut comp_max_sim: BTreeMap<usize, f64> = BTreeMap::new();
    let mut comp_pair_count: BTreeMap<usize, usize> = BTreeMap::new();
    for pair in pairs {
        let a = name_to_idx[pair.left.as_str()];
        let root = ds.find(a);
        let entry = comp_max_sim.entry(root).or_insert(0.0);
        if pair.similarity > *entry {
            *entry = pair.similarity;
        }
        *comp_pair_count.entry(root).or_insert(0) += 1;
    }

    let mut clusters: Vec<NearDupCluster> = components
        .into_iter()
        .map(|(root, members)| {
            let mut file_list: Vec<String> =
                members.iter().map(|&i| names[i].to_string()).collect();
            file_list.sort();

            // Representative: most-connected file, tie-break alphabetical
            let representative = members
                .iter()
                .copied()
                .max_by(|&a, &b| {
                    let ca = connection_count.get(&a).copied().unwrap_or(0);
                    let cb = connection_count.get(&b).copied().unwrap_or(0);
                    ca.cmp(&cb).then_with(|| names[b].cmp(names[a]))
                })
                .map(|i| names[i].to_string())
                .unwrap_or_default();

            let max_similarity = comp_max_sim.get(&root).copied().unwrap_or(0.0);
            let pair_count = comp_pair_count.get(&root).copied().unwrap_or(0);

            NearDupCluster {
                files: file_list,
                max_similarity,
                representative,
                pair_count,
            }
        })
        .collect();

    // Sort clusters by max_similarity desc, then representative name
    clusters.sort_by(|a, b| {
        b.max_similarity
            .partial_cmp(&a.max_similarity)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.representative.cmp(&b.representative))
    });

    clusters
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

    // ---------------------------------------------------------------
    // DisjointSets tests
    // ---------------------------------------------------------------

    #[test]
    fn disjoint_sets_find_self() {
        let mut ds = DisjointSets::new(5);
        for i in 0..5 {
            assert_eq!(ds.find(i), i);
        }
    }

    #[test]
    fn disjoint_sets_union_and_find() {
        let mut ds = DisjointSets::new(5);
        ds.union(0, 1);
        ds.union(2, 3);
        assert_eq!(ds.find(0), ds.find(1));
        assert_eq!(ds.find(2), ds.find(3));
        assert_ne!(ds.find(0), ds.find(2));
        // Merge the two groups
        ds.union(1, 3);
        assert_eq!(ds.find(0), ds.find(3));
    }

    #[test]
    fn disjoint_sets_idempotent_union() {
        let mut ds = DisjointSets::new(3);
        ds.union(0, 1);
        ds.union(0, 1); // repeated
        assert_eq!(ds.find(0), ds.find(1));
    }

    // ---------------------------------------------------------------
    // build_clusters tests
    // ---------------------------------------------------------------

    #[test]
    fn build_clusters_empty() {
        let pairs: Vec<NearDupPairRow> = vec![];
        let clusters = build_clusters(&pairs);
        assert!(clusters.is_empty());
    }

    #[test]
    fn build_clusters_single_pair() {
        let pairs = vec![NearDupPairRow {
            left: "a.rs".to_string(),
            right: "b.rs".to_string(),
            similarity: 0.95,
            shared_fingerprints: 10,
            left_fingerprints: 20,
            right_fingerprints: 20,
        }];
        let clusters = build_clusters(&pairs);
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].files, vec!["a.rs", "b.rs"]);
        assert!((clusters[0].max_similarity - 0.95).abs() < 1e-10);
        assert_eq!(clusters[0].pair_count, 1);
    }

    #[test]
    fn build_clusters_two_components() {
        let pairs = vec![
            NearDupPairRow {
                left: "a.rs".to_string(),
                right: "b.rs".to_string(),
                similarity: 0.90,
                shared_fingerprints: 10,
                left_fingerprints: 20,
                right_fingerprints: 20,
            },
            NearDupPairRow {
                left: "c.rs".to_string(),
                right: "d.rs".to_string(),
                similarity: 0.85,
                shared_fingerprints: 8,
                left_fingerprints: 20,
                right_fingerprints: 20,
            },
        ];
        let clusters = build_clusters(&pairs);
        assert_eq!(clusters.len(), 2);
        // Sorted by max_similarity desc
        assert!((clusters[0].max_similarity - 0.90).abs() < 1e-10);
        assert!((clusters[1].max_similarity - 0.85).abs() < 1e-10);
    }

    #[test]
    fn build_clusters_merged_component() {
        // Three files forming a triangle: all in one component
        let pairs = vec![
            NearDupPairRow {
                left: "a.rs".to_string(),
                right: "b.rs".to_string(),
                similarity: 0.90,
                shared_fingerprints: 10,
                left_fingerprints: 20,
                right_fingerprints: 20,
            },
            NearDupPairRow {
                left: "b.rs".to_string(),
                right: "c.rs".to_string(),
                similarity: 0.85,
                shared_fingerprints: 8,
                left_fingerprints: 20,
                right_fingerprints: 20,
            },
            NearDupPairRow {
                left: "a.rs".to_string(),
                right: "c.rs".to_string(),
                similarity: 0.80,
                shared_fingerprints: 7,
                left_fingerprints: 20,
                right_fingerprints: 20,
            },
        ];
        let clusters = build_clusters(&pairs);
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].files, vec!["a.rs", "b.rs", "c.rs"]);
        assert!((clusters[0].max_similarity - 0.90).abs() < 1e-10);
        assert_eq!(clusters[0].pair_count, 3);
        // "a.rs" and "b.rs" each have 2 connections; "a.rs" wins alphabetically
        assert_eq!(clusters[0].representative, "a.rs");
    }

    #[test]
    fn build_clusters_representative_most_connected() {
        // b.rs is connected to both a.rs and c.rs, but a.rs and c.rs are not connected
        let pairs = vec![
            NearDupPairRow {
                left: "a.rs".to_string(),
                right: "b.rs".to_string(),
                similarity: 0.90,
                shared_fingerprints: 10,
                left_fingerprints: 20,
                right_fingerprints: 20,
            },
            NearDupPairRow {
                left: "b.rs".to_string(),
                right: "c.rs".to_string(),
                similarity: 0.85,
                shared_fingerprints: 8,
                left_fingerprints: 20,
                right_fingerprints: 20,
            },
        ];
        let clusters = build_clusters(&pairs);
        assert_eq!(clusters.len(), 1);
        // b.rs has 2 connections, a.rs and c.rs have 1 each
        assert_eq!(clusters[0].representative, "b.rs");
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
    fn clusters_complete_despite_truncation() {
        // Build 3 pairs across 2 components, truncate to 2 pairs.
        // Clusters should still reflect all 3 pairs.
        let pairs = vec![
            NearDupPairRow {
                left: "a.rs".to_string(),
                right: "b.rs".to_string(),
                similarity: 0.95,
                shared_fingerprints: 10,
                left_fingerprints: 20,
                right_fingerprints: 20,
            },
            NearDupPairRow {
                left: "c.rs".to_string(),
                right: "d.rs".to_string(),
                similarity: 0.90,
                shared_fingerprints: 9,
                left_fingerprints: 20,
                right_fingerprints: 20,
            },
            NearDupPairRow {
                left: "d.rs".to_string(),
                right: "e.rs".to_string(),
                similarity: 0.85,
                shared_fingerprints: 8,
                left_fingerprints: 20,
                right_fingerprints: 20,
            },
        ];

        // Build clusters from ALL pairs (before truncation)
        let clusters = build_clusters(&pairs);
        // 2 components: {a,b} and {c,d,e}
        assert_eq!(clusters.len(), 2);

        // The {c,d,e} cluster should have 3 files and 2 pairs
        let large_cluster = clusters.iter().find(|c| c.files.len() == 3).unwrap();
        assert_eq!(large_cluster.pair_count, 2);
        assert_eq!(large_cluster.files, vec!["c.rs", "d.rs", "e.rs"]);
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
