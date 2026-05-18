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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_tool() -> ToolInfo {
        ToolInfo {
            name: "tokmd".into(),
            version: "1.0.0".into(),
        }
    }

    fn sample_context_file_row() -> ContextFileRow {
        ContextFileRow {
            path: "src/main.rs".into(),
            module: "src".into(),
            lang: "Rust".into(),
            tokens: 100,
            code: 50,
            lines: 65,
            bytes: 2_000,
            value: 75,
            rank_reason: String::new(),
            policy: InclusionPolicy::Full,
            effective_tokens: None,
            policy_reason: None,
            classifications: vec![],
        }
    }

    // ── Schema version constants ────────────────────────────────────
    #[test]
    fn handoff_schema_version_constant() {
        assert_eq!(HANDOFF_SCHEMA_VERSION, 5);
    }

    #[test]
    fn context_bundle_schema_version_constant() {
        assert_eq!(CONTEXT_BUNDLE_SCHEMA_VERSION, 2);
    }

    #[test]
    fn context_schema_version_constant() {
        assert_eq!(CONTEXT_SCHEMA_VERSION, 4);
    }

    // ── ContextReceipt ──────────────────────────────────────────────
    #[test]
    fn context_receipt_serde_roundtrip_minimal() {
        let receipt = ContextReceipt {
            schema_version: CONTEXT_SCHEMA_VERSION,
            generated_at_ms: 1_700_000_000_000,
            tool: sample_tool(),
            mode: "context".into(),
            budget_tokens: 8_000,
            used_tokens: 4_000,
            utilization_pct: 50.0,
            strategy: "value-greedy".into(),
            rank_by: "value".into(),
            file_count: 1,
            files: vec![sample_context_file_row()],
            rank_by_effective: None,
            fallback_reason: None,
            excluded_by_policy: vec![],
            token_estimation: None,
            bundle_audit: None,
        };
        let json = serde_json::to_string(&receipt).unwrap();
        let back: ContextReceipt = serde_json::from_str(&json).unwrap();
        assert_eq!(back.schema_version, CONTEXT_SCHEMA_VERSION);
        assert_eq!(back.mode, "context");
        assert_eq!(back.files.len(), 1);
        assert_eq!(back.budget_tokens, 8_000);
    }

    #[test]
    fn context_receipt_omits_optional_when_empty_or_none() {
        let receipt = ContextReceipt {
            schema_version: CONTEXT_SCHEMA_VERSION,
            generated_at_ms: 0,
            tool: sample_tool(),
            mode: "context".into(),
            budget_tokens: 0,
            used_tokens: 0,
            utilization_pct: 0.0,
            strategy: String::new(),
            rank_by: String::new(),
            file_count: 0,
            files: vec![],
            rank_by_effective: None,
            fallback_reason: None,
            excluded_by_policy: vec![],
            token_estimation: None,
            bundle_audit: None,
        };
        let value = serde_json::to_value(&receipt).unwrap();
        assert!(value.get("rank_by_effective").is_none());
        assert!(value.get("fallback_reason").is_none());
        assert!(value.get("excluded_by_policy").is_none());
        assert!(value.get("token_estimation").is_none());
        assert!(value.get("bundle_audit").is_none());
    }

    #[test]
    fn context_receipt_field_names_stable() {
        let receipt = ContextReceipt {
            schema_version: CONTEXT_SCHEMA_VERSION,
            generated_at_ms: 0,
            tool: sample_tool(),
            mode: "context".into(),
            budget_tokens: 100,
            used_tokens: 50,
            utilization_pct: 0.5,
            strategy: "s".into(),
            rank_by: "r".into(),
            file_count: 0,
            files: vec![],
            rank_by_effective: None,
            fallback_reason: None,
            excluded_by_policy: vec![],
            token_estimation: None,
            bundle_audit: None,
        };
        let value = serde_json::to_value(&receipt).unwrap();
        for key in [
            "schema_version",
            "generated_at_ms",
            "tool",
            "mode",
            "budget_tokens",
            "used_tokens",
            "utilization_pct",
            "strategy",
            "rank_by",
            "file_count",
            "files",
        ] {
            assert!(
                value.get(key).is_some(),
                "missing key `{key}` in ContextReceipt"
            );
        }
    }

    // ── ContextFileRow ──────────────────────────────────────────────
    #[test]
    fn context_file_row_omits_default_policy() {
        let row = sample_context_file_row();
        let value = serde_json::to_value(&row).unwrap();
        assert!(value.get("policy").is_none());
        assert!(value.get("rank_reason").is_none());
        assert!(value.get("effective_tokens").is_none());
        assert!(value.get("policy_reason").is_none());
        assert!(value.get("classifications").is_none());
    }

    #[test]
    fn context_file_row_keeps_non_default_fields() {
        let row = ContextFileRow {
            policy: InclusionPolicy::HeadTail,
            effective_tokens: Some(40),
            policy_reason: Some("budget".into()),
            rank_reason: "high churn".into(),
            classifications: vec![FileClassification::Generated],
            ..sample_context_file_row()
        };
        let value = serde_json::to_value(&row).unwrap();
        assert_eq!(value["policy"], "head_tail");
        assert_eq!(value["effective_tokens"], 40);
        assert_eq!(value["policy_reason"], "budget");
        assert_eq!(value["rank_reason"], "high churn");
        assert_eq!(value["classifications"][0], "generated");
    }

    #[test]
    fn context_file_row_serde_roundtrip() {
        let row = sample_context_file_row();
        let json = serde_json::to_string(&row).unwrap();
        let back: ContextFileRow = serde_json::from_str(&json).unwrap();
        assert_eq!(back.path, row.path);
        assert_eq!(back.tokens, row.tokens);
        assert_eq!(back.policy, row.policy);
    }

    // ── ContextLogRecord ────────────────────────────────────────────
    #[test]
    fn context_log_record_serde_roundtrip() {
        let rec = ContextLogRecord {
            schema_version: CONTEXT_SCHEMA_VERSION,
            generated_at_ms: 1_700_000_000_000,
            tool: sample_tool(),
            budget_tokens: 10_000,
            used_tokens: 9_000,
            utilization_pct: 90.0,
            strategy: "greedy".into(),
            rank_by: "value".into(),
            file_count: 12,
            total_bytes: 35_000,
            output_destination: "stdout".into(),
        };
        let json = serde_json::to_string(&rec).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        for key in [
            "schema_version",
            "generated_at_ms",
            "tool",
            "budget_tokens",
            "used_tokens",
            "utilization_pct",
            "strategy",
            "rank_by",
            "file_count",
            "total_bytes",
            "output_destination",
        ] {
            assert!(
                value.get(key).is_some(),
                "missing key `{key}` in ContextLogRecord"
            );
        }
        let back: ContextLogRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(back.file_count, 12);
        assert_eq!(back.output_destination, "stdout");
    }

    // ── TokenEstimationMeta ─────────────────────────────────────────
    #[test]
    fn token_estimation_default_divisor_constants() {
        assert_eq!(TokenEstimationMeta::DEFAULT_BPT_EST, 4.0);
        assert_eq!(TokenEstimationMeta::DEFAULT_BPT_LOW, 3.0);
        assert_eq!(TokenEstimationMeta::DEFAULT_BPT_HIGH, 5.0);
    }

    #[test]
    fn token_estimation_invariant_min_le_est_le_max() {
        let est = TokenEstimationMeta::from_bytes(12_345, 4.0);
        assert!(est.tokens_min <= est.tokens_est);
        assert!(est.tokens_est <= est.tokens_max);
    }

    #[test]
    fn token_estimation_serde_roundtrip_keeps_invariant() {
        let est = TokenEstimationMeta::from_bytes_with_bounds(10_000, 4.0, 3.0, 5.0);
        let json = serde_json::to_string(&est).unwrap();
        let back: TokenEstimationMeta = serde_json::from_str(&json).unwrap();
        assert_eq!(back.source_bytes, 10_000);
        assert_eq!(back.bytes_per_token_est, 4.0);
        assert_eq!(back.tokens_min, est.tokens_min);
        assert_eq!(back.tokens_est, est.tokens_est);
        assert_eq!(back.tokens_max, est.tokens_max);
        assert!(back.tokens_min <= back.tokens_est);
        assert!(back.tokens_est <= back.tokens_max);
    }

    #[test]
    fn token_estimation_accepts_legacy_aliases() {
        let json = r#"{
            "bytes_per_token_est": 4.0,
            "bytes_per_token_low": 3.0,
            "bytes_per_token_high": 5.0,
            "tokens_high": 200,
            "tokens_est": 250,
            "tokens_low": 333,
            "source_bytes": 1000
        }"#;
        let est: TokenEstimationMeta = serde_json::from_str(json).unwrap();
        assert_eq!(est.tokens_min, 200);
        assert_eq!(est.tokens_est, 250);
        assert_eq!(est.tokens_max, 333);
    }

    // ── TokenAudit ──────────────────────────────────────────────────
    #[test]
    fn token_audit_serde_roundtrip() {
        let audit = TokenAudit::from_output(5_000, 4_500);
        let json = serde_json::to_string(&audit).unwrap();
        let back: TokenAudit = serde_json::from_str(&json).unwrap();
        assert_eq!(back.output_bytes, 5_000);
        assert_eq!(back.overhead_bytes, 500);
        assert!(back.tokens_min <= back.tokens_est);
        assert!(back.tokens_est <= back.tokens_max);
    }

    #[test]
    fn token_audit_accepts_legacy_aliases() {
        let json = r#"{
            "output_bytes": 1000,
            "tokens_high": 200,
            "tokens_est": 250,
            "tokens_low": 333,
            "overhead_bytes": 100,
            "overhead_pct": 0.1
        }"#;
        let audit: TokenAudit = serde_json::from_str(json).unwrap();
        assert_eq!(audit.tokens_min, 200);
        assert_eq!(audit.tokens_max, 333);
    }

    // ── FileClassification ──────────────────────────────────────────
    #[test]
    fn file_classification_uses_snake_case() {
        assert_eq!(
            serde_json::to_string(&FileClassification::DataBlob).unwrap(),
            "\"data_blob\""
        );
        assert_eq!(
            serde_json::to_string(&FileClassification::Sourcemap).unwrap(),
            "\"sourcemap\""
        );
    }

    #[test]
    fn file_classification_all_variants_roundtrip() {
        for variant in [
            FileClassification::Generated,
            FileClassification::Fixture,
            FileClassification::Vendored,
            FileClassification::Lockfile,
            FileClassification::Minified,
            FileClassification::DataBlob,
            FileClassification::Sourcemap,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: FileClassification = serde_json::from_str(&json).unwrap();
            assert_eq!(back, variant);
        }
    }

    #[test]
    fn file_classification_ord_is_stable() {
        let mut variants = [
            FileClassification::Sourcemap,
            FileClassification::Generated,
            FileClassification::Lockfile,
        ];
        variants.sort();
        assert_eq!(variants[0], FileClassification::Generated);
        assert_eq!(variants[1], FileClassification::Lockfile);
        assert_eq!(variants[2], FileClassification::Sourcemap);
    }

    // ── InclusionPolicy ─────────────────────────────────────────────
    #[test]
    fn inclusion_policy_default_is_full() {
        assert_eq!(InclusionPolicy::default(), InclusionPolicy::Full);
    }

    #[test]
    fn inclusion_policy_uses_snake_case() {
        assert_eq!(
            serde_json::to_string(&InclusionPolicy::HeadTail).unwrap(),
            "\"head_tail\""
        );
        assert_eq!(
            serde_json::to_string(&InclusionPolicy::Full).unwrap(),
            "\"full\""
        );
    }

    #[test]
    fn inclusion_policy_all_variants_roundtrip() {
        for variant in [
            InclusionPolicy::Full,
            InclusionPolicy::HeadTail,
            InclusionPolicy::Summary,
            InclusionPolicy::Skip,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: InclusionPolicy = serde_json::from_str(&json).unwrap();
            assert_eq!(back, variant);
        }
    }

    #[test]
    fn is_default_policy_helper_only_true_for_full() {
        assert!(is_default_policy(&InclusionPolicy::Full));
        assert!(!is_default_policy(&InclusionPolicy::HeadTail));
        assert!(!is_default_policy(&InclusionPolicy::Summary));
        assert!(!is_default_policy(&InclusionPolicy::Skip));
    }

    // ── PolicyExcludedFile ──────────────────────────────────────────
    #[test]
    fn policy_excluded_file_serde_roundtrip() {
        let f = PolicyExcludedFile {
            path: "vendor/big.json".into(),
            original_tokens: 10_000,
            policy: InclusionPolicy::Skip,
            reason: "data_blob".into(),
            classifications: vec![FileClassification::DataBlob, FileClassification::Vendored],
        };
        let json = serde_json::to_string(&f).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["policy"], "skip");
        assert_eq!(value["classifications"][0], "data_blob");
        assert_eq!(value["classifications"][1], "vendored");
        let back: PolicyExcludedFile = serde_json::from_str(&json).unwrap();
        assert_eq!(back.path, f.path);
        assert_eq!(back.policy, f.policy);
    }

    // ── HandoffManifest ─────────────────────────────────────────────
    #[test]
    fn handoff_manifest_minimal_serde_roundtrip() {
        let m = HandoffManifest {
            schema_version: HANDOFF_SCHEMA_VERSION,
            generated_at_ms: 1_700_000_000_000,
            tool: sample_tool(),
            mode: "handoff".into(),
            inputs: vec![".".into()],
            output_dir: "/tmp/out".into(),
            budget_tokens: 8_000,
            used_tokens: 4_000,
            utilization_pct: 50.0,
            strategy: "value-greedy".into(),
            rank_by: "value".into(),
            capabilities: vec![CapabilityStatus {
                name: "git".into(),
                status: CapabilityState::Available,
                reason: None,
            }],
            artifacts: vec![ArtifactEntry {
                name: "summary.md".into(),
                path: "out/summary.md".into(),
                description: "Markdown summary".into(),
                bytes: 256,
                hash: None,
            }],
            included_files: vec![sample_context_file_row()],
            excluded_paths: vec![],
            excluded_patterns: vec![],
            smart_excluded_files: vec![],
            total_files: 10,
            bundled_files: 5,
            intelligence_preset: "compact".into(),
            rank_by_effective: None,
            fallback_reason: None,
            excluded_by_policy: vec![],
            token_estimation: None,
            code_audit: None,
        };
        let json = serde_json::to_string(&m).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        for key in [
            "schema_version",
            "generated_at_ms",
            "tool",
            "mode",
            "inputs",
            "output_dir",
            "budget_tokens",
            "used_tokens",
            "utilization_pct",
            "strategy",
            "rank_by",
            "capabilities",
            "artifacts",
            "included_files",
            "excluded_paths",
            "excluded_patterns",
            "smart_excluded_files",
            "total_files",
            "bundled_files",
            "intelligence_preset",
        ] {
            assert!(
                value.get(key).is_some(),
                "missing key `{key}` in HandoffManifest"
            );
        }
        assert!(value.get("rank_by_effective").is_none());
        assert!(value.get("fallback_reason").is_none());
        assert!(value.get("excluded_by_policy").is_none());
        assert!(value.get("token_estimation").is_none());
        assert!(value.get("code_audit").is_none());
        let back: HandoffManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.schema_version, HANDOFF_SCHEMA_VERSION);
        assert_eq!(back.bundled_files, 5);
        assert_eq!(back.intelligence_preset, "compact");
    }

    // ── ContextBundleManifest ───────────────────────────────────────
    #[test]
    fn context_bundle_manifest_serde_roundtrip() {
        let m = ContextBundleManifest {
            schema_version: CONTEXT_BUNDLE_SCHEMA_VERSION,
            generated_at_ms: 0,
            tool: sample_tool(),
            mode: "context".into(),
            budget_tokens: 0,
            used_tokens: 0,
            utilization_pct: 0.0,
            strategy: "value".into(),
            rank_by: "value".into(),
            file_count: 0,
            bundle_bytes: 0,
            artifacts: vec![],
            included_files: vec![],
            excluded_paths: vec![ContextExcludedPath {
                path: "secret".into(),
                reason: "redacted".into(),
            }],
            excluded_patterns: vec!["target".into()],
            rank_by_effective: None,
            fallback_reason: None,
            excluded_by_policy: vec![],
            token_estimation: None,
            bundle_audit: None,
        };
        let json = serde_json::to_string(&m).unwrap();
        let back: ContextBundleManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.excluded_paths.len(), 1);
        assert_eq!(back.excluded_paths[0].path, "secret");
        assert_eq!(back.excluded_patterns, vec!["target".to_string()]);
    }

    // ── ContextExcludedPath / HandoffExcludedPath / SmartExcludedFile ──
    #[test]
    fn context_excluded_path_serde_roundtrip() {
        let v = ContextExcludedPath {
            path: "p".into(),
            reason: "r".into(),
        };
        let json = serde_json::to_string(&v).unwrap();
        let back: ContextExcludedPath = serde_json::from_str(&json).unwrap();
        assert_eq!(back.path, "p");
        assert_eq!(back.reason, "r");
    }

    #[test]
    fn handoff_excluded_path_serde_roundtrip() {
        let v = HandoffExcludedPath {
            path: "p".into(),
            reason: "r".into(),
        };
        let json = serde_json::to_string(&v).unwrap();
        let back: HandoffExcludedPath = serde_json::from_str(&json).unwrap();
        assert_eq!(back.path, "p");
        assert_eq!(back.reason, "r");
    }

    #[test]
    fn smart_excluded_file_serde_roundtrip() {
        let v = SmartExcludedFile {
            path: "vendor/x.min.js".into(),
            reason: "minified".into(),
            tokens: 999,
        };
        let json = serde_json::to_string(&v).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        for key in ["path", "reason", "tokens"] {
            assert!(value.get(key).is_some());
        }
        let back: SmartExcludedFile = serde_json::from_str(&json).unwrap();
        assert_eq!(back.tokens, 999);
    }

    // ── HandoffIntelligence + sub-types ─────────────────────────────
    #[test]
    fn handoff_intelligence_full_roundtrip() {
        let intel = HandoffIntelligence {
            tree: Some("root\n  a\n  b".into()),
            tree_depth: Some(2),
            hotspots: Some(vec![HandoffHotspot {
                path: "src/main.rs".into(),
                commits: 10,
                lines: 100,
                score: 42,
            }]),
            complexity: Some(HandoffComplexity {
                total_functions: 100,
                avg_function_length: 12.5,
                max_function_length: 80,
                avg_cyclomatic: 3.5,
                max_cyclomatic: 30,
                high_risk_files: 2,
            }),
            derived: Some(HandoffDerived {
                total_files: 50,
                total_code: 5_000,
                total_lines: 6_500,
                total_tokens: 12_000,
                lang_count: 3,
                dominant_lang: "Rust".into(),
                dominant_pct: 80.0,
            }),
            warnings: vec!["no git".into()],
        };
        let json = serde_json::to_string(&intel).unwrap();
        let back: HandoffIntelligence = serde_json::from_str(&json).unwrap();
        assert_eq!(back.tree_depth, Some(2));
        let hotspots = back.hotspots.expect("hotspots present");
        assert_eq!(hotspots.len(), 1);
        assert_eq!(hotspots[0].score, 42);
        let complexity = back.complexity.expect("complexity present");
        assert_eq!(complexity.high_risk_files, 2);
        let derived = back.derived.expect("derived present");
        assert_eq!(derived.dominant_lang, "Rust");
        assert_eq!(back.warnings, vec!["no git".to_string()]);
    }

    #[test]
    fn handoff_intelligence_all_none_serializes() {
        let intel = HandoffIntelligence {
            tree: None,
            tree_depth: None,
            hotspots: None,
            complexity: None,
            derived: None,
            warnings: vec![],
        };
        let json = serde_json::to_string(&intel).unwrap();
        let back: HandoffIntelligence = serde_json::from_str(&json).unwrap();
        assert!(back.tree.is_none());
        assert!(back.hotspots.is_none());
        assert!(back.warnings.is_empty());
    }

    // ── CapabilityState / CapabilityStatus ──────────────────────────
    #[test]
    fn capability_state_uses_snake_case() {
        assert_eq!(
            serde_json::to_string(&CapabilityState::Available).unwrap(),
            "\"available\""
        );
        assert_eq!(
            serde_json::to_string(&CapabilityState::Skipped).unwrap(),
            "\"skipped\""
        );
        assert_eq!(
            serde_json::to_string(&CapabilityState::Unavailable).unwrap(),
            "\"unavailable\""
        );
    }

    #[test]
    fn capability_state_all_variants_roundtrip() {
        for variant in [
            CapabilityState::Available,
            CapabilityState::Skipped,
            CapabilityState::Unavailable,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: CapabilityState = serde_json::from_str(&json).unwrap();
            assert_eq!(back, variant);
        }
    }

    #[test]
    fn capability_status_with_reason_keeps_field() {
        let s = CapabilityStatus {
            name: "git".into(),
            status: CapabilityState::Unavailable,
            reason: Some("not a repo".into()),
        };
        let value = serde_json::to_value(&s).unwrap();
        assert_eq!(value["reason"], "not a repo");
        let back: CapabilityStatus = serde_json::from_str(&value.to_string()).unwrap();
        assert_eq!(back.reason.as_deref(), Some("not a repo"));
    }

    #[test]
    fn capability_status_without_reason_omits_field() {
        let s = CapabilityStatus {
            name: "git".into(),
            status: CapabilityState::Available,
            reason: None,
        };
        let value = serde_json::to_value(&s).unwrap();
        assert!(value.get("reason").is_none());
    }

    // ── ArtifactEntry / ArtifactHash ────────────────────────────────
    #[test]
    fn artifact_entry_with_hash_roundtrip() {
        let a = ArtifactEntry {
            name: "bundle.txt".into(),
            path: "out/bundle.txt".into(),
            description: "Concatenated source".into(),
            bytes: 1_024,
            hash: Some(ArtifactHash {
                algo: "sha256".into(),
                hash: "deadbeef".into(),
            }),
        };
        let json = serde_json::to_string(&a).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        for key in ["name", "path", "description", "bytes", "hash"] {
            assert!(
                value.get(key).is_some(),
                "missing key `{key}` in ArtifactEntry"
            );
        }
        assert_eq!(value["hash"]["algo"], "sha256");
        let back: ArtifactEntry = serde_json::from_str(&json).unwrap();
        let h = back.hash.expect("hash present");
        assert_eq!(h.algo, "sha256");
        assert_eq!(h.hash, "deadbeef");
    }

    #[test]
    fn artifact_entry_without_hash_omits_field() {
        let a = ArtifactEntry {
            name: "x".into(),
            path: "y".into(),
            description: "z".into(),
            bytes: 0,
            hash: None,
        };
        let value = serde_json::to_value(&a).unwrap();
        assert!(value.get("hash").is_none());
    }
}
