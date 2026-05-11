//! Context and handoff receipt DTOs.
//!
//! This module owns the serde-stable context packing and handoff artifact
//! contracts. Public consumers should keep using the root-level re-exports
//! from `tokmd_types`.

use serde::{Deserialize, Serialize};

use crate::ToolInfo;

/// Schema version for handoff receipts.
///
/// ```
/// assert_eq!(tokmd_types::HANDOFF_SCHEMA_VERSION, 5);
/// ```
pub const HANDOFF_SCHEMA_VERSION: u32 = 5;

/// Schema version for context bundle manifests.
///
/// ```
/// assert_eq!(tokmd_types::CONTEXT_BUNDLE_SCHEMA_VERSION, 2);
/// ```
pub const CONTEXT_BUNDLE_SCHEMA_VERSION: u32 = 2;

/// Schema version for context receipts (separate from SCHEMA_VERSION used by lang/module/export/diff).
///
/// ```
/// assert_eq!(tokmd_types::CONTEXT_SCHEMA_VERSION, 4);
/// ```
pub const CONTEXT_SCHEMA_VERSION: u32 = 4;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextReceipt {
    pub schema_version: u32,
    pub generated_at_ms: u128,
    pub tool: ToolInfo,
    pub mode: String,
    pub budget_tokens: usize,
    pub used_tokens: usize,
    pub utilization_pct: f64,
    pub strategy: String,
    pub rank_by: String,
    pub file_count: usize,
    pub files: Vec<ContextFileRow>,
    /// Effective ranking metric (may differ from rank_by if fallback occurred).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rank_by_effective: Option<String>,
    /// Reason for fallback if rank_by_effective differs from rank_by.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback_reason: Option<String>,
    /// Files excluded by per-file cap / classification policy.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub excluded_by_policy: Vec<PolicyExcludedFile>,
    /// Token estimation envelope with uncertainty bounds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_estimation: Option<TokenEstimationMeta>,
    /// Post-bundle audit comparing actual bytes to estimates.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bundle_audit: Option<TokenAudit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextFileRow {
    pub path: String,
    pub module: String,
    pub lang: String,
    pub tokens: usize,
    pub code: usize,
    pub lines: usize,
    pub bytes: usize,
    pub value: usize,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub rank_reason: String,
    /// Inclusion policy applied to this file.
    #[serde(default, skip_serializing_if = "is_default_policy")]
    pub policy: InclusionPolicy,
    /// Effective token count when policy != Full (None means same as `tokens`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effective_tokens: Option<usize>,
    /// Reason for the applied policy.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_reason: Option<String>,
    /// File classifications detected by hygiene analysis.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub classifications: Vec<FileClassification>,
}

/// Log record for context command JSONL append mode.
/// Contains metadata only (not file contents) for lightweight logging.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextLogRecord {
    pub schema_version: u32,
    pub generated_at_ms: u128,
    pub tool: ToolInfo,
    pub budget_tokens: usize,
    pub used_tokens: usize,
    pub utilization_pct: f64,
    pub strategy: String,
    pub rank_by: String,
    pub file_count: usize,
    pub total_bytes: usize,
    pub output_destination: String,
}

/// Metadata about how token estimates were produced.
///
/// Rails are NOT guaranteed bounds - they are heuristic fences.
/// Default divisors: est=4.0, low=3.0 (conservative -> more tokens),
/// high=5.0 (optimistic -> fewer tokens).
///
/// **Invariant**: `tokens_min <= tokens_est <= tokens_max`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenEstimationMeta {
    /// Divisor used for main estimate (default 4.0).
    pub bytes_per_token_est: f64,
    /// Conservative divisor - more tokens (default 3.0).
    pub bytes_per_token_low: f64,
    /// Optimistic divisor - fewer tokens (default 5.0).
    pub bytes_per_token_high: f64,
    /// tokens = source_bytes / bytes_per_token_high (optimistic, fewest tokens).
    #[serde(alias = "tokens_high")]
    pub tokens_min: usize,
    /// tokens = source_bytes / bytes_per_token_est.
    pub tokens_est: usize,
    /// tokens = source_bytes / bytes_per_token_low (conservative, most tokens).
    #[serde(alias = "tokens_low")]
    pub tokens_max: usize,
    /// Total source bytes used to compute estimates.
    pub source_bytes: usize,
}

impl TokenEstimationMeta {
    /// Default bytes-per-token divisors.
    pub const DEFAULT_BPT_EST: f64 = 4.0;
    pub const DEFAULT_BPT_LOW: f64 = 3.0;
    pub const DEFAULT_BPT_HIGH: f64 = 5.0;

    /// Create estimation from source byte count using default divisors.
    ///
    /// # Examples
    ///
    /// ```
    /// use tokmd_types::TokenEstimationMeta;
    ///
    /// let est = TokenEstimationMeta::from_bytes(4000, 4.0);
    /// assert_eq!(est.tokens_est, 1000);
    /// assert_eq!(est.source_bytes, 4000);
    /// // Invariant: tokens_min <= tokens_est <= tokens_max
    /// assert!(est.tokens_min <= est.tokens_est);
    /// assert!(est.tokens_est <= est.tokens_max);
    /// ```
    pub fn from_bytes(bytes: usize, bpt: f64) -> Self {
        Self::from_bytes_with_bounds(bytes, bpt, Self::DEFAULT_BPT_LOW, Self::DEFAULT_BPT_HIGH)
    }

    /// Create estimation from source byte count with explicit low/high divisors.
    pub fn from_bytes_with_bounds(bytes: usize, bpt_est: f64, bpt_low: f64, bpt_high: f64) -> Self {
        Self {
            bytes_per_token_est: bpt_est,
            bytes_per_token_low: bpt_low,
            bytes_per_token_high: bpt_high,
            tokens_min: (bytes as f64 / bpt_high).ceil() as usize,
            tokens_est: (bytes as f64 / bpt_est).ceil() as usize,
            tokens_max: (bytes as f64 / bpt_low).ceil() as usize,
            source_bytes: bytes,
        }
    }
}

/// Post-write audit comparing actual output to estimates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenAudit {
    /// Actual bytes written to the output bundle.
    pub output_bytes: u64,
    /// tokens = output_bytes / bytes_per_token_high (optimistic, fewest tokens).
    #[serde(alias = "tokens_high")]
    pub tokens_min: usize,
    /// tokens = output_bytes / bytes_per_token_est.
    pub tokens_est: usize,
    /// tokens = output_bytes / bytes_per_token_low (conservative, most tokens).
    #[serde(alias = "tokens_low")]
    pub tokens_max: usize,
    /// Bytes of framing/separators/headers (output_bytes - content_bytes).
    pub overhead_bytes: u64,
    /// overhead_bytes / output_bytes (0.0-1.0).
    pub overhead_pct: f64,
}

impl TokenAudit {
    /// Create an audit from output bytes and content bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use tokmd_types::TokenAudit;
    ///
    /// let audit = TokenAudit::from_output(5000, 4500);
    /// assert_eq!(audit.output_bytes, 5000);
    /// assert_eq!(audit.overhead_bytes, 500);
    /// assert!(audit.overhead_pct > 0.0);
    /// ```
    pub fn from_output(output_bytes: u64, content_bytes: u64) -> Self {
        Self::from_output_with_divisors(
            output_bytes,
            content_bytes,
            TokenEstimationMeta::DEFAULT_BPT_EST,
            TokenEstimationMeta::DEFAULT_BPT_LOW,
            TokenEstimationMeta::DEFAULT_BPT_HIGH,
        )
    }

    /// Create an audit from output bytes with explicit divisors.
    pub fn from_output_with_divisors(
        output_bytes: u64,
        content_bytes: u64,
        bpt_est: f64,
        bpt_low: f64,
        bpt_high: f64,
    ) -> Self {
        let overhead_bytes = output_bytes.saturating_sub(content_bytes);
        let overhead_pct = if output_bytes > 0 {
            overhead_bytes as f64 / output_bytes as f64
        } else {
            0.0
        };
        Self {
            output_bytes,
            tokens_min: (output_bytes as f64 / bpt_high).ceil() as usize,
            tokens_est: (output_bytes as f64 / bpt_est).ceil() as usize,
            tokens_max: (output_bytes as f64 / bpt_low).ceil() as usize,
            overhead_bytes,
            overhead_pct,
        }
    }
}

/// Classification of a file for bundle hygiene purposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileClassification {
    /// Protobuf output, parser tables, node-types.json, etc.
    Generated,
    /// Test fixtures, golden snapshots.
    Fixture,
    /// Third-party vendored code.
    Vendored,
    /// Cargo.lock, package-lock.json, etc.
    Lockfile,
    /// *.min.js, *.min.css.
    Minified,
    /// Files with very high tokens-per-line ratio.
    DataBlob,
    /// *.js.map, *.css.map.
    Sourcemap,
}

/// How a file is included in the context/handoff bundle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum InclusionPolicy {
    /// Full file content.
    #[default]
    Full,
    /// First N + last N lines.
    HeadTail,
    /// Structural summary (placeholder, behaves as Skip for now).
    Summary,
    /// Excluded from payload entirely.
    Skip,
}

/// Helper for serde skip_serializing_if on InclusionPolicy.
pub(crate) fn is_default_policy(policy: &InclusionPolicy) -> bool {
    *policy == InclusionPolicy::Full
}

/// A file excluded by per-file cap / classification policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyExcludedFile {
    pub path: String,
    pub original_tokens: usize,
    pub policy: InclusionPolicy,
    pub reason: String,
    pub classifications: Vec<FileClassification>,
}

/// Manifest for a handoff bundle containing LLM-ready artifacts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffManifest {
    pub schema_version: u32,
    pub generated_at_ms: u128,
    pub tool: ToolInfo,
    pub mode: String,
    pub inputs: Vec<String>,
    pub output_dir: String,
    pub budget_tokens: usize,
    pub used_tokens: usize,
    pub utilization_pct: f64,
    pub strategy: String,
    pub rank_by: String,
    pub capabilities: Vec<CapabilityStatus>,
    pub artifacts: Vec<ArtifactEntry>,
    pub included_files: Vec<ContextFileRow>,
    pub excluded_paths: Vec<HandoffExcludedPath>,
    pub excluded_patterns: Vec<String>,
    pub smart_excluded_files: Vec<SmartExcludedFile>,
    pub total_files: usize,
    pub bundled_files: usize,
    pub intelligence_preset: String,
    /// Effective ranking metric (may differ from rank_by if fallback occurred).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rank_by_effective: Option<String>,
    /// Reason for fallback if rank_by_effective differs from rank_by.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback_reason: Option<String>,
    /// Files excluded by per-file cap / classification policy.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub excluded_by_policy: Vec<PolicyExcludedFile>,
    /// Token estimation envelope with uncertainty bounds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_estimation: Option<TokenEstimationMeta>,
    /// Post-bundle audit comparing actual code bundle bytes to estimates.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code_audit: Option<TokenAudit>,
}

/// A file excluded by smart-exclude heuristics (lockfiles, minified, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartExcludedFile {
    pub path: String,
    pub reason: String,
    pub tokens: usize,
}

/// Manifest for a context bundle directory (bundle.txt + receipt.json + manifest.json).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextBundleManifest {
    pub schema_version: u32,
    pub generated_at_ms: u128,
    pub tool: ToolInfo,
    pub mode: String,
    pub budget_tokens: usize,
    pub used_tokens: usize,
    pub utilization_pct: f64,
    pub strategy: String,
    pub rank_by: String,
    pub file_count: usize,
    pub bundle_bytes: usize,
    pub artifacts: Vec<ArtifactEntry>,
    pub included_files: Vec<ContextFileRow>,
    pub excluded_paths: Vec<ContextExcludedPath>,
    pub excluded_patterns: Vec<String>,
    /// Effective ranking metric (may differ from rank_by if fallback occurred).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rank_by_effective: Option<String>,
    /// Reason for fallback if rank_by_effective differs from rank_by.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback_reason: Option<String>,
    /// Files excluded by per-file cap / classification policy.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub excluded_by_policy: Vec<PolicyExcludedFile>,
    /// Token estimation envelope with uncertainty bounds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_estimation: Option<TokenEstimationMeta>,
    /// Post-bundle audit comparing actual bundle bytes to estimates.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bundle_audit: Option<TokenAudit>,
}

/// Explicitly excluded path with reason for context bundles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextExcludedPath {
    pub path: String,
    pub reason: String,
}

/// Intelligence bundle for handoff containing tree, hotspots, complexity, and derived metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffIntelligence {
    pub tree: Option<String>,
    pub tree_depth: Option<usize>,
    pub hotspots: Option<Vec<HandoffHotspot>>,
    pub complexity: Option<HandoffComplexity>,
    pub derived: Option<HandoffDerived>,
    pub warnings: Vec<String>,
}

/// Explicitly excluded path with reason.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffExcludedPath {
    pub path: String,
    pub reason: String,
}

/// Simplified hotspot row for handoff intelligence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffHotspot {
    pub path: String,
    pub commits: usize,
    pub lines: usize,
    pub score: usize,
}

/// Simplified complexity report for handoff intelligence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffComplexity {
    pub total_functions: usize,
    pub avg_function_length: f64,
    pub max_function_length: usize,
    pub avg_cyclomatic: f64,
    pub max_cyclomatic: usize,
    pub high_risk_files: usize,
}

/// Simplified derived metrics for handoff intelligence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffDerived {
    pub total_files: usize,
    pub total_code: usize,
    pub total_lines: usize,
    pub total_tokens: usize,
    pub lang_count: usize,
    pub dominant_lang: String,
    pub dominant_pct: f64,
}

/// Status of a detected capability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityStatus {
    pub name: String,
    pub status: CapabilityState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// State of a capability: available, skipped, or unavailable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityState {
    /// Capability is available and was used.
    Available,
    /// Capability is available but was skipped (e.g., --no-git flag).
    Skipped,
    /// Capability is unavailable (e.g., not in a git repo).
    Unavailable,
}

/// Entry describing an artifact in the handoff bundle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactEntry {
    pub name: String,
    pub path: String,
    pub description: String,
    pub bytes: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<ArtifactHash>,
}

/// Hash for artifact integrity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactHash {
    pub algo: String,
    pub hash: String,
}
