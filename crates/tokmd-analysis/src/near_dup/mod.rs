//! Near-duplicate detection via Winnowing fingerprinting.
//!
//! Implements a content-based near-duplicate detection algorithm:
//! 1. Tokenize source text by splitting on non-alphanumeric boundaries
//! 2. Build k-grams (k=25 tokens) and hash each with FxHash
//! 3. Apply Winnowing (window size w=4) to select representative fingerprints
//! 4. Build inverted index from fingerprints to files
//! 5. Compute Jaccard similarity for candidate pairs
//! 6. Emit pairs exceeding the similarity threshold

use std::path::Path;

use anyhow::Result;

use tokmd_analysis_types::{
    NearDupAlgorithm, NearDupParams, NearDupScope, NearDupStats, NearDuplicateReport,
};
use tokmd_types::ExportData;

mod clusters;
mod fingerprint;
mod pairs;
mod selection;
use clusters::build_clusters;
use fingerprint::{K, MAX_POSTINGS, W, read_and_fingerprint};
use pairs::build_pairs;
use selection::{SelectedFiles, partition_files, select_files};

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
    let SelectedFiles {
        files,
        eligible_files,
        files_skipped,
        excluded_by_pattern,
        max_file_bytes,
    } = select_files(export, max_files, limits, exclude_patterns)?;

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

    let files_analyzed = files.len();

    // Partition files by scope
    let partitions = partition_files(&files, scope);

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
    // Phase 2: Pairing
    let pairing = build_pairs(&partition_fps, &files, threshold);
    let mut all_pairs = pairing.pairs;
    let pairing_ms = pairing.pairing_ms;

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
