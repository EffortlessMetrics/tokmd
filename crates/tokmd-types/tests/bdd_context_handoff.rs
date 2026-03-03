//! BDD-style scenario tests for context and handoff receipt types.
//!
//! Each test follows Given/When/Then structure to verify construction,
//! serialization, and behavioural invariants of context packing and
//! handoff bundle types.

use tokmd_types::{
    ArtifactEntry, ArtifactHash, CONTEXT_BUNDLE_SCHEMA_VERSION, CONTEXT_SCHEMA_VERSION,
    CapabilityState, CapabilityStatus, ContextBundleManifest, ContextExcludedPath, ContextFileRow,
    ContextLogRecord, ContextReceipt, FileClassification, HANDOFF_SCHEMA_VERSION,
    HandoffComplexity, HandoffDerived, HandoffExcludedPath, HandoffHotspot, HandoffIntelligence,
    HandoffManifest, InclusionPolicy, PolicyExcludedFile, SmartExcludedFile, TokenAudit,
    TokenEstimationMeta, ToolInfo,
};

// ── helpers ────────────────────────────────────────────────────────────────

fn tool_info() -> ToolInfo {
    ToolInfo {
        name: "tokmd".to_string(),
        version: "0.1.0-test".to_string(),
    }
}

fn sample_context_file_row(path: &str, tokens: usize) -> ContextFileRow {
    ContextFileRow {
        path: path.to_string(),
        module: "src".to_string(),
        lang: "Rust".to_string(),
        tokens,
        code: tokens / 2,
        lines: tokens,
        bytes: tokens * 4,
        value: tokens,
        rank_reason: "code".to_string(),
        policy: InclusionPolicy::Full,
        effective_tokens: None,
        policy_reason: None,
        classifications: vec![],
    }
}

// =============================================================================
// Scenario: ContextReceipt construction and budget utilisation
// =============================================================================

#[test]
fn given_context_receipt_with_files_when_constructed_then_schema_version_matches_constant() {
    let receipt = ContextReceipt {
        schema_version: CONTEXT_SCHEMA_VERSION,
        generated_at_ms: 1_700_000_000_000,
        tool: tool_info(),
        mode: "greedy".to_string(),
        budget_tokens: 128_000,
        used_tokens: 64_000,
        utilization_pct: 50.0,
        strategy: "greedy".to_string(),
        rank_by: "code".to_string(),
        file_count: 2,
        files: vec![
            sample_context_file_row("src/main.rs", 30_000),
            sample_context_file_row("src/lib.rs", 34_000),
        ],
        rank_by_effective: None,
        fallback_reason: None,
        excluded_by_policy: vec![],
        token_estimation: None,
        bundle_audit: None,
    };

    assert_eq!(receipt.schema_version, CONTEXT_SCHEMA_VERSION);
    assert_eq!(receipt.file_count, 2);
    assert_eq!(receipt.files.len(), 2);
}

#[test]
fn given_context_receipt_when_serialized_then_json_contains_budget_fields() {
    let receipt = ContextReceipt {
        schema_version: CONTEXT_SCHEMA_VERSION,
        generated_at_ms: 1_700_000_000_000,
        tool: tool_info(),
        mode: "greedy".to_string(),
        budget_tokens: 128_000,
        used_tokens: 100_000,
        utilization_pct: 78.125,
        strategy: "greedy".to_string(),
        rank_by: "code".to_string(),
        file_count: 1,
        files: vec![sample_context_file_row("src/main.rs", 100_000)],
        rank_by_effective: None,
        fallback_reason: None,
        excluded_by_policy: vec![],
        token_estimation: None,
        bundle_audit: None,
    };

    let json = serde_json::to_string(&receipt).unwrap();
    assert!(json.contains("\"budget_tokens\":128000"));
    assert!(json.contains("\"used_tokens\":100000"));
    assert!(json.contains("\"utilization_pct\":78.125"));
}

#[test]
fn given_context_receipt_with_policy_exclusions_when_serialized_then_exclusions_present() {
    let receipt = ContextReceipt {
        schema_version: CONTEXT_SCHEMA_VERSION,
        generated_at_ms: 0,
        tool: tool_info(),
        mode: "greedy".to_string(),
        budget_tokens: 10_000,
        used_tokens: 0,
        utilization_pct: 0.0,
        strategy: "greedy".to_string(),
        rank_by: "code".to_string(),
        file_count: 0,
        files: vec![],
        rank_by_effective: None,
        fallback_reason: None,
        excluded_by_policy: vec![PolicyExcludedFile {
            path: "vendor/huge.js".to_string(),
            original_tokens: 50_000,
            policy: InclusionPolicy::Skip,
            reason: "vendored file exceeds cap".to_string(),
            classifications: vec![FileClassification::Vendored],
        }],
        token_estimation: None,
        bundle_audit: None,
    };

    let json = serde_json::to_string(&receipt).unwrap();
    assert!(json.contains("\"excluded_by_policy\""));
    assert!(json.contains("vendor/huge.js"));
}

// =============================================================================
// Scenario: ContextLogRecord for lightweight JSONL logging
// =============================================================================

#[test]
fn given_context_log_record_when_serialized_then_roundtrips_correctly() {
    let record = ContextLogRecord {
        schema_version: CONTEXT_SCHEMA_VERSION,
        generated_at_ms: 1_700_000_000_000,
        tool: tool_info(),
        budget_tokens: 128_000,
        used_tokens: 100_000,
        utilization_pct: 78.125,
        strategy: "greedy".to_string(),
        rank_by: "code".to_string(),
        file_count: 42,
        total_bytes: 500_000,
        output_destination: "stdout".to_string(),
    };

    let json = serde_json::to_string(&record).unwrap();
    let parsed: ContextLogRecord = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.file_count, 42);
    assert_eq!(parsed.total_bytes, 500_000);
    assert_eq!(parsed.output_destination, "stdout");
}

// =============================================================================
// Scenario: ContextBundleManifest construction
// =============================================================================

#[test]
fn given_context_bundle_manifest_when_constructed_then_schema_version_matches_constant() {
    let manifest = ContextBundleManifest {
        schema_version: CONTEXT_BUNDLE_SCHEMA_VERSION,
        generated_at_ms: 1_700_000_000_000,
        tool: tool_info(),
        mode: "greedy".to_string(),
        budget_tokens: 128_000,
        used_tokens: 80_000,
        utilization_pct: 62.5,
        strategy: "greedy".to_string(),
        rank_by: "code".to_string(),
        file_count: 3,
        bundle_bytes: 320_000,
        artifacts: vec![ArtifactEntry {
            name: "bundle.txt".to_string(),
            path: "output/bundle.txt".to_string(),
            description: "Concatenated source files".to_string(),
            bytes: 320_000,
            hash: Some(ArtifactHash {
                algo: "blake3".to_string(),
                hash: "abc123".to_string(),
            }),
        }],
        included_files: vec![sample_context_file_row("src/main.rs", 80_000)],
        excluded_paths: vec![ContextExcludedPath {
            path: "target/".to_string(),
            reason: "build output".to_string(),
        }],
        excluded_patterns: vec!["*.lock".to_string()],
        rank_by_effective: None,
        fallback_reason: None,
        excluded_by_policy: vec![],
        token_estimation: None,
        bundle_audit: None,
    };

    assert_eq!(manifest.schema_version, CONTEXT_BUNDLE_SCHEMA_VERSION);
    assert_eq!(manifest.artifacts.len(), 1);
    assert_eq!(manifest.excluded_paths.len(), 1);
    assert_eq!(manifest.excluded_patterns, vec!["*.lock"]);
}

// =============================================================================
// Scenario: HandoffManifest construction and serialization
// =============================================================================

#[test]
fn given_handoff_manifest_when_constructed_then_schema_version_matches_constant() {
    let manifest = HandoffManifest {
        schema_version: HANDOFF_SCHEMA_VERSION,
        generated_at_ms: 1_700_000_000_000,
        tool: tool_info(),
        mode: "greedy".to_string(),
        inputs: vec![".".to_string()],
        output_dir: "handoff-output".to_string(),
        budget_tokens: 200_000,
        used_tokens: 150_000,
        utilization_pct: 75.0,
        strategy: "greedy".to_string(),
        rank_by: "code".to_string(),
        capabilities: vec![
            CapabilityStatus {
                name: "git".to_string(),
                status: CapabilityState::Available,
                reason: None,
            },
            CapabilityStatus {
                name: "content".to_string(),
                status: CapabilityState::Skipped,
                reason: Some("--no-content flag".to_string()),
            },
        ],
        artifacts: vec![],
        included_files: vec![sample_context_file_row("src/lib.rs", 50_000)],
        excluded_paths: vec![HandoffExcludedPath {
            path: "target/".to_string(),
            reason: "build artifacts".to_string(),
        }],
        excluded_patterns: vec![],
        smart_excluded_files: vec![SmartExcludedFile {
            path: "Cargo.lock".to_string(),
            reason: "lockfile".to_string(),
            tokens: 25_000,
        }],
        total_files: 100,
        bundled_files: 80,
        intelligence_preset: "health".to_string(),
        rank_by_effective: None,
        fallback_reason: None,
        excluded_by_policy: vec![],
        token_estimation: None,
        code_audit: None,
    };

    assert_eq!(manifest.schema_version, HANDOFF_SCHEMA_VERSION);
    assert_eq!(manifest.capabilities.len(), 2);
    assert_eq!(manifest.smart_excluded_files.len(), 1);
    assert_eq!(manifest.total_files, 100);
    assert_eq!(manifest.bundled_files, 80);
}

#[test]
fn given_handoff_manifest_when_serialized_then_json_roundtrips() {
    let manifest = HandoffManifest {
        schema_version: HANDOFF_SCHEMA_VERSION,
        generated_at_ms: 0,
        tool: tool_info(),
        mode: "greedy".to_string(),
        inputs: vec![".".to_string()],
        output_dir: "out".to_string(),
        budget_tokens: 100_000,
        used_tokens: 50_000,
        utilization_pct: 50.0,
        strategy: "greedy".to_string(),
        rank_by: "code".to_string(),
        capabilities: vec![],
        artifacts: vec![],
        included_files: vec![],
        excluded_paths: vec![],
        excluded_patterns: vec![],
        smart_excluded_files: vec![],
        total_files: 0,
        bundled_files: 0,
        intelligence_preset: "receipt".to_string(),
        rank_by_effective: Some("tokens".to_string()),
        fallback_reason: Some("no git history".to_string()),
        excluded_by_policy: vec![],
        token_estimation: None,
        code_audit: None,
    };

    let json = serde_json::to_string(&manifest).unwrap();
    let parsed: HandoffManifest = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.rank_by_effective.as_deref(), Some("tokens"));
    assert_eq!(parsed.fallback_reason.as_deref(), Some("no git history"));
    assert_eq!(parsed.intelligence_preset, "receipt");
}

// =============================================================================
// Scenario: HandoffIntelligence with and without optional sections
// =============================================================================

#[test]
fn given_handoff_intelligence_with_all_sections_when_constructed_then_sections_accessible() {
    let intel = HandoffIntelligence {
        tree: Some("├── src/\n│   └── main.rs".to_string()),
        tree_depth: Some(2),
        hotspots: Some(vec![
            HandoffHotspot {
                path: "src/main.rs".to_string(),
                commits: 50,
                lines: 200,
                score: 10_000,
            },
            HandoffHotspot {
                path: "src/lib.rs".to_string(),
                commits: 10,
                lines: 100,
                score: 1_000,
            },
        ]),
        complexity: Some(HandoffComplexity {
            total_functions: 120,
            avg_function_length: 15.5,
            max_function_length: 80,
            avg_cyclomatic: 3.2,
            max_cyclomatic: 25,
            high_risk_files: 4,
        }),
        derived: Some(HandoffDerived {
            total_files: 50,
            total_code: 10_000,
            total_lines: 15_000,
            total_tokens: 25_000,
            lang_count: 3,
            dominant_lang: "Rust".to_string(),
            dominant_pct: 85.5,
        }),
        warnings: vec!["high complexity in src/parser.rs".to_string()],
    };

    assert!(intel.tree.is_some());
    assert_eq!(intel.tree_depth, Some(2));
    assert_eq!(intel.hotspots.as_ref().unwrap().len(), 2);
    assert_eq!(intel.complexity.as_ref().unwrap().total_functions, 120);
    assert_eq!(intel.derived.as_ref().unwrap().dominant_lang, "Rust");
    assert_eq!(intel.warnings.len(), 1);
}

#[test]
fn given_handoff_intelligence_with_no_optional_data_when_constructed_then_all_none() {
    let intel = HandoffIntelligence {
        tree: None,
        tree_depth: None,
        hotspots: None,
        complexity: None,
        derived: None,
        warnings: vec![],
    };

    assert!(intel.tree.is_none());
    assert!(intel.hotspots.is_none());
    assert!(intel.complexity.is_none());
    assert!(intel.derived.is_none());
    assert!(intel.warnings.is_empty());
}

// =============================================================================
// Scenario: HandoffDerived dominant language invariants
// =============================================================================

#[test]
fn given_handoff_derived_when_dominant_pct_set_then_within_zero_to_hundred() {
    let derived = HandoffDerived {
        total_files: 10,
        total_code: 1_000,
        total_lines: 1_500,
        total_tokens: 2_500,
        lang_count: 2,
        dominant_lang: "Python".to_string(),
        dominant_pct: 60.0,
    };

    assert!(
        derived.dominant_pct >= 0.0 && derived.dominant_pct <= 100.0,
        "dominant_pct should be 0..100, got {}",
        derived.dominant_pct
    );
    assert!(derived.lang_count >= 1);
}

// =============================================================================
// Scenario: Token budget enforcement through utilisation tracking
// =============================================================================

#[test]
fn given_receipt_where_used_exceeds_budget_when_checked_then_utilization_over_100() {
    // Context packing can overshoot by one file. The receipt faithfully records this.
    let receipt = ContextReceipt {
        schema_version: CONTEXT_SCHEMA_VERSION,
        generated_at_ms: 0,
        tool: tool_info(),
        mode: "greedy".to_string(),
        budget_tokens: 10_000,
        used_tokens: 10_500,
        utilization_pct: 105.0,
        strategy: "greedy".to_string(),
        rank_by: "code".to_string(),
        file_count: 1,
        files: vec![sample_context_file_row("huge.rs", 10_500)],
        rank_by_effective: None,
        fallback_reason: None,
        excluded_by_policy: vec![],
        token_estimation: None,
        bundle_audit: None,
    };

    assert!(receipt.used_tokens > receipt.budget_tokens);
    assert!(receipt.utilization_pct > 100.0);
}

#[test]
fn given_empty_receipt_when_budget_unused_then_utilization_is_zero() {
    let receipt = ContextReceipt {
        schema_version: CONTEXT_SCHEMA_VERSION,
        generated_at_ms: 0,
        tool: tool_info(),
        mode: "greedy".to_string(),
        budget_tokens: 128_000,
        used_tokens: 0,
        utilization_pct: 0.0,
        strategy: "greedy".to_string(),
        rank_by: "code".to_string(),
        file_count: 0,
        files: vec![],
        rank_by_effective: None,
        fallback_reason: None,
        excluded_by_policy: vec![],
        token_estimation: None,
        bundle_audit: None,
    };

    assert_eq!(receipt.used_tokens, 0);
    assert_eq!(receipt.file_count, 0);
    assert!((receipt.utilization_pct - 0.0).abs() < f64::EPSILON);
}

// =============================================================================
// Scenario: ContextFileRow selection with mixed inclusion policies
// =============================================================================

#[test]
fn given_files_with_mixed_policies_when_inspected_then_policies_correctly_assigned() {
    let full_file = ContextFileRow {
        path: "src/main.rs".to_string(),
        module: "src".to_string(),
        lang: "Rust".to_string(),
        tokens: 500,
        code: 250,
        lines: 400,
        bytes: 2_000,
        value: 250,
        rank_reason: "code".to_string(),
        policy: InclusionPolicy::Full,
        effective_tokens: None,
        policy_reason: None,
        classifications: vec![],
    };

    let head_tail_file = ContextFileRow {
        path: "src/huge.rs".to_string(),
        module: "src".to_string(),
        lang: "Rust".to_string(),
        tokens: 50_000,
        code: 25_000,
        lines: 40_000,
        bytes: 200_000,
        value: 25_000,
        rank_reason: "code".to_string(),
        policy: InclusionPolicy::HeadTail,
        effective_tokens: Some(4_000),
        policy_reason: Some("file exceeds cap".to_string()),
        classifications: vec![],
    };

    let skipped_file = ContextFileRow {
        path: "vendor/dep.min.js".to_string(),
        module: "vendor".to_string(),
        lang: "JavaScript".to_string(),
        tokens: 100_000,
        code: 1,
        lines: 1,
        bytes: 400_000,
        value: 0,
        rank_reason: String::new(),
        policy: InclusionPolicy::Skip,
        effective_tokens: Some(0),
        policy_reason: Some("vendored minified file".to_string()),
        classifications: vec![FileClassification::Vendored, FileClassification::Minified],
    };

    // Full policy: effective_tokens is None (same as tokens)
    assert_eq!(full_file.policy, InclusionPolicy::Full);
    assert!(full_file.effective_tokens.is_none());

    // HeadTail policy: effective_tokens < tokens
    assert_eq!(head_tail_file.policy, InclusionPolicy::HeadTail);
    assert!(head_tail_file.effective_tokens.unwrap() < head_tail_file.tokens);

    // Skip policy: effective_tokens is zero
    assert_eq!(skipped_file.policy, InclusionPolicy::Skip);
    assert_eq!(skipped_file.effective_tokens, Some(0));
    assert_eq!(skipped_file.classifications.len(), 2);
}

// =============================================================================
// Scenario: HandoffManifest with token estimation and audit
// =============================================================================

#[test]
fn given_handoff_manifest_with_estimation_and_audit_when_serialized_then_fields_present() {
    let estimation = TokenEstimationMeta::from_bytes(400_000, TokenEstimationMeta::DEFAULT_BPT_EST);
    let audit = TokenAudit::from_output(420_000, 400_000);

    let manifest = HandoffManifest {
        schema_version: HANDOFF_SCHEMA_VERSION,
        generated_at_ms: 0,
        tool: tool_info(),
        mode: "greedy".to_string(),
        inputs: vec![".".to_string()],
        output_dir: "out".to_string(),
        budget_tokens: 200_000,
        used_tokens: 100_000,
        utilization_pct: 50.0,
        strategy: "greedy".to_string(),
        rank_by: "code".to_string(),
        capabilities: vec![],
        artifacts: vec![],
        included_files: vec![],
        excluded_paths: vec![],
        excluded_patterns: vec![],
        smart_excluded_files: vec![],
        total_files: 0,
        bundled_files: 0,
        intelligence_preset: "deep".to_string(),
        rank_by_effective: None,
        fallback_reason: None,
        excluded_by_policy: vec![],
        token_estimation: Some(estimation),
        code_audit: Some(audit),
    };

    let json = serde_json::to_string(&manifest).unwrap();
    assert!(json.contains("\"token_estimation\""));
    assert!(json.contains("\"code_audit\""));
    assert!(json.contains("\"bytes_per_token_est\""));
    assert!(json.contains("\"overhead_pct\""));

    let parsed: HandoffManifest = serde_json::from_str(&json).unwrap();
    let est = parsed.token_estimation.unwrap();
    assert!(est.tokens_min <= est.tokens_est);
    assert!(est.tokens_est <= est.tokens_max);
}
