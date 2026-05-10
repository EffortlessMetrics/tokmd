//! Duplicate and near-duplicate receipt DTOs.
//!
//! These contract types remain re-exported from the crate root to preserve
//! existing `tokmd_analysis_types::...` names.

use serde::{Deserialize, Serialize};

// ----------------------------
// Near-duplicate detection
// ----------------------------

/// Scope for near-duplicate comparison partitioning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum NearDupScope {
    /// Compare files within the same module.
    #[default]
    Module,
    /// Compare files within the same language.
    Lang,
    /// Compare all files globally.
    Global,
}

/// Parameters for near-duplicate detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NearDupParams {
    pub scope: NearDupScope,
    pub threshold: f64,
    pub max_files: usize,
    /// Maximum pairs to emit (truncation guardrail).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_pairs: Option<usize>,
    /// Effective per-file byte limit used for eligibility filtering.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_file_bytes: Option<u64>,
    /// How files were selected for analysis.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selection_method: Option<String>,
    /// Algorithm constants used for fingerprinting.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub algorithm: Option<NearDupAlgorithm>,
    /// Glob patterns used to exclude files from near-dup analysis.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude_patterns: Vec<String>,
}

/// Algorithm constants for near-duplicate fingerprinting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NearDupAlgorithm {
    /// Number of tokens per k-gram shingle.
    pub k_gram_size: usize,
    /// Winnowing window size.
    pub window_size: usize,
    /// Skip fingerprints appearing in more than this many files.
    pub max_postings: usize,
}

/// Report of near-duplicate file pairs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NearDuplicateReport {
    pub params: NearDupParams,
    pub pairs: Vec<NearDupPairRow>,
    pub files_analyzed: usize,
    pub files_skipped: usize,
    /// Number of files eligible before the max_files cap.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eligible_files: Option<usize>,
    /// Connected-component clusters derived from pairs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clusters: Option<Vec<NearDupCluster>>,
    /// Whether the pairs list was truncated by `max_pairs`.
    /// Clusters are built from the complete pair set before truncation.
    #[serde(default)]
    pub truncated: bool,
    /// Number of files excluded by glob patterns.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub excluded_by_pattern: Option<usize>,
    /// Runtime performance statistics.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stats: Option<NearDupStats>,
}

/// A connected component of near-duplicate files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NearDupCluster {
    /// Files in this cluster, sorted alphabetically.
    pub files: Vec<String>,
    /// Maximum pairwise similarity in the cluster.
    pub max_similarity: f64,
    /// Most-connected file (tie-break alphabetical).
    pub representative: String,
    /// Number of pairs within this cluster.
    pub pair_count: usize,
}

/// Runtime statistics for near-duplicate detection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NearDupStats {
    /// Time spent computing fingerprints (milliseconds).
    pub fingerprinting_ms: u64,
    /// Time spent computing pair similarities (milliseconds).
    pub pairing_ms: u64,
    /// Total bytes of source files processed.
    pub bytes_processed: u64,
}

/// A pair of near-duplicate files with similarity score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NearDupPairRow {
    pub left: String,
    pub right: String,
    pub similarity: f64,
    pub shared_fingerprints: usize,
    pub left_fingerprints: usize,
    pub right_fingerprints: usize,
}

// -------------------
// Duplication metrics
// -------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateReport {
    pub groups: Vec<DuplicateGroup>,
    pub wasted_bytes: u64,
    pub strategy: String,
    /// Duplication density summary overall and by module.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub density: Option<DuplicationDensityReport>,
    /// Near-duplicate file pairs detected by fingerprint similarity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub near: Option<NearDuplicateReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateGroup {
    pub hash: String,
    pub bytes: u64,
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicationDensityReport {
    pub duplicate_groups: usize,
    pub duplicate_files: usize,
    pub duplicated_bytes: u64,
    pub wasted_bytes: u64,
    pub wasted_pct_of_codebase: f64,
    pub by_module: Vec<ModuleDuplicationDensityRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleDuplicationDensityRow {
    pub module: String,
    pub duplicate_files: usize,
    pub wasted_files: usize,
    pub duplicated_bytes: u64,
    pub wasted_bytes: u64,
    pub module_bytes: u64,
    pub density: f64,
}
