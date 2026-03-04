//! BDD-style integration tests for handoff and context receipt types.
//!
//! These tests verify HandoffManifest construction, serialization,
//! intelligence preset handling, token budget partitioning,
//! context receipts, and schema version correctness.

use serde_json::Value;
use tokmd_types::{
    ArtifactEntry, ArtifactHash, CONTEXT_BUNDLE_SCHEMA_VERSION, CONTEXT_SCHEMA_VERSION,
    CapabilityState, CapabilityStatus, ContextBundleManifest, ContextExcludedPath, ContextFileRow,
    ContextLogRecord, ContextReceipt, FileClassification, HANDOFF_SCHEMA_VERSION,
    HandoffComplexity, HandoffDerived, HandoffExcludedPath, HandoffHotspot, HandoffIntelligence,
    HandoffManifest, InclusionPolicy, PolicyExcludedFile, SmartExcludedFile, TokenAudit,
    TokenEstimationMeta, ToolInfo,
};

// =============================================================================
// Helpers
// =============================================================================

fn sample_tool_info() -> ToolInfo {
    ToolInfo {
        name: "tokmd".to_string(),
        version: "0.0.0-test".to_string(),
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

fn sample_handoff_manifest() -> HandoffManifest {
    HandoffManifest {
        schema_version: HANDOFF_SCHEMA_VERSION,
        generated_at_ms: 1700000000000,
        tool: sample_tool_info(),
        mode: "handoff".to_string(),
        inputs: vec![".".to_string()],
        output_dir: "handoff_output".to_string(),
        budget_tokens: 100_000,
        used_tokens: 85_000,
        utilization_pct: 85.0,
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
                status: CapabilityState::Unavailable,
                reason: Some("feature not enabled".to_string()),
            },
        ],
        artifacts: vec![
            ArtifactEntry {
                name: "code.txt".to_string(),
                path: "handoff_output/code.txt".to_string(),
                description: "Source code bundle".to_string(),
                bytes: 340_000,
                hash: Some(ArtifactHash {
                    algo: "blake3".to_string(),
                    hash: "abcdef1234567890".to_string(),
                }),
            },
            ArtifactEntry {
                name: "manifest.json".to_string(),
                path: "handoff_output/manifest.json".to_string(),
                description: "Bundle manifest".to_string(),
                bytes: 2_000,
                hash: None,
            },
        ],
        included_files: vec![
            sample_context_file_row("src/lib.rs", 5000),
            sample_context_file_row("src/main.rs", 3000),
        ],
        excluded_paths: vec![HandoffExcludedPath {
            path: "target/".to_string(),
            reason: "build output".to_string(),
        }],
        excluded_patterns: vec!["*.lock".to_string()],
        smart_excluded_files: vec![SmartExcludedFile {
            path: "Cargo.lock".to_string(),
            reason: "lockfile".to_string(),
            tokens: 120_000,
        }],
        total_files: 50,
        bundled_files: 30,
        intelligence_preset: "standard".to_string(),
        rank_by_effective: None,
        fallback_reason: None,
        excluded_by_policy: vec![],
        token_estimation: None,
        code_audit: None,
    }
}

fn sample_context_receipt() -> ContextReceipt {
    ContextReceipt {
        schema_version: CONTEXT_SCHEMA_VERSION,
        generated_at_ms: 1700000000000,
        tool: sample_tool_info(),
        mode: "context".to_string(),
        budget_tokens: 50_000,
        used_tokens: 42_000,
        utilization_pct: 84.0,
        strategy: "greedy".to_string(),
        rank_by: "code".to_string(),
        file_count: 15,
        files: vec![
            sample_context_file_row("src/lib.rs", 5000),
            sample_context_file_row("src/main.rs", 3000),
        ],
        rank_by_effective: None,
        fallback_reason: None,
        excluded_by_policy: vec![],
        token_estimation: None,
        bundle_audit: None,
    }
}

fn sample_context_bundle_manifest() -> ContextBundleManifest {
    ContextBundleManifest {
        schema_version: CONTEXT_BUNDLE_SCHEMA_VERSION,
        generated_at_ms: 1700000000000,
        tool: sample_tool_info(),
        mode: "context-bundle".to_string(),
        budget_tokens: 50_000,
        used_tokens: 45_000,
        utilization_pct: 90.0,
        strategy: "greedy".to_string(),
        rank_by: "code".to_string(),
        file_count: 20,
        bundle_bytes: 180_000,
        artifacts: vec![ArtifactEntry {
            name: "bundle.txt".to_string(),
            path: "context_bundle/bundle.txt".to_string(),
            description: "Context bundle".to_string(),
            bytes: 180_000,
            hash: Some(ArtifactHash {
                algo: "blake3".to_string(),
                hash: "fedcba9876543210".to_string(),
            }),
        }],
        included_files: vec![sample_context_file_row("src/lib.rs", 5000)],
        excluded_paths: vec![ContextExcludedPath {
            path: "node_modules/".to_string(),
            reason: "vendored dependencies".to_string(),
        }],
        excluded_patterns: vec!["*.min.js".to_string()],
        rank_by_effective: None,
        fallback_reason: None,
        excluded_by_policy: vec![],
        token_estimation: None,
        bundle_audit: None,
    }
}

// =============================================================================
// Scenario: Schema version constants are correct
// =============================================================================

#[test]
fn given_handoff_schema_version_then_it_equals_five() {
    assert_eq!(HANDOFF_SCHEMA_VERSION, 5);
}

#[test]
fn given_context_schema_version_then_it_equals_four() {
    assert_eq!(CONTEXT_SCHEMA_VERSION, 4);
}

#[test]
fn given_context_bundle_schema_version_then_it_equals_two() {
    assert_eq!(CONTEXT_BUNDLE_SCHEMA_VERSION, 2);
}

// =============================================================================
// Scenario: HandoffManifest construction and field access
// =============================================================================

#[test]
fn given_handoff_manifest_then_all_required_fields_accessible() {
    let manifest = sample_handoff_manifest();

    assert_eq!(manifest.schema_version, HANDOFF_SCHEMA_VERSION);
    assert_eq!(manifest.mode, "handoff");
    assert_eq!(manifest.budget_tokens, 100_000);
    assert_eq!(manifest.used_tokens, 85_000);
    assert_eq!(manifest.strategy, "greedy");
    assert_eq!(manifest.rank_by, "code");
    assert_eq!(manifest.total_files, 50);
    assert_eq!(manifest.bundled_files, 30);
    assert_eq!(manifest.intelligence_preset, "standard");
    assert_eq!(manifest.capabilities.len(), 2);
    assert_eq!(manifest.artifacts.len(), 2);
    assert_eq!(manifest.included_files.len(), 2);
}

// =============================================================================
// Scenario: HandoffManifest JSON serialization
// =============================================================================

#[test]
fn given_handoff_manifest_when_serialized_then_json_has_required_fields() {
    let manifest = sample_handoff_manifest();
    let json: Value = serde_json::to_value(manifest).unwrap();

    assert_eq!(json["schema_version"], HANDOFF_SCHEMA_VERSION);
    assert!(json["generated_at_ms"].is_number());
    assert!(json["tool"].is_object());
    assert_eq!(json["mode"], "handoff");
    assert!(json["inputs"].is_array());
    assert!(json["output_dir"].is_string());
    assert!(json["budget_tokens"].is_number());
    assert!(json["used_tokens"].is_number());
    assert!(json["utilization_pct"].is_number());
    assert!(json["strategy"].is_string());
    assert!(json["rank_by"].is_string());
    assert!(json["capabilities"].is_array());
    assert!(json["artifacts"].is_array());
    assert!(json["included_files"].is_array());
    assert!(json["excluded_paths"].is_array());
    assert!(json["excluded_patterns"].is_array());
    assert!(json["smart_excluded_files"].is_array());
    assert!(json["total_files"].is_number());
    assert!(json["bundled_files"].is_number());
    assert!(json["intelligence_preset"].is_string());
}

#[test]
fn given_handoff_manifest_when_optional_fields_empty_then_omitted_from_json() {
    let manifest = sample_handoff_manifest();
    let json = serde_json::to_string(&manifest).unwrap();

    assert!(!json.contains("\"rank_by_effective\""));
    assert!(!json.contains("\"fallback_reason\""));
    assert!(!json.contains("\"excluded_by_policy\""));
    assert!(!json.contains("\"token_estimation\""));
    assert!(!json.contains("\"code_audit\""));
}

#[test]
fn given_handoff_manifest_with_optional_fields_then_present_in_json() {
    let mut manifest = sample_handoff_manifest();
    manifest.rank_by_effective = Some("churn".to_string());
    manifest.fallback_reason = Some("git not available".to_string());
    manifest.excluded_by_policy = vec![PolicyExcludedFile {
        path: "gen/proto.rs".to_string(),
        original_tokens: 50_000,
        policy: InclusionPolicy::Skip,
        reason: "generated code".to_string(),
        classifications: vec![FileClassification::Generated],
    }];
    manifest.token_estimation = Some(TokenEstimationMeta::from_bytes(
        340_000,
        TokenEstimationMeta::DEFAULT_BPT_EST,
    ));
    manifest.code_audit = Some(TokenAudit::from_output(340_000, 320_000));

    let json: Value = serde_json::to_value(manifest).unwrap();
    assert_eq!(json["rank_by_effective"], "churn");
    assert_eq!(json["fallback_reason"], "git not available");
    assert!(json["excluded_by_policy"].is_array());
    assert_eq!(json["excluded_by_policy"][0]["path"], "gen/proto.rs");
    assert!(json["token_estimation"].is_object());
    assert!(json["code_audit"].is_object());
}

// =============================================================================
// Scenario: HandoffManifest serde roundtrip
// =============================================================================

#[test]
fn given_handoff_manifest_when_roundtripped_then_data_preserved() {
    let manifest = sample_handoff_manifest();
    let json_str = serde_json::to_string(&manifest).unwrap();
    let back: HandoffManifest = serde_json::from_str(&json_str).unwrap();

    assert_eq!(back.schema_version, manifest.schema_version);
    assert_eq!(back.mode, manifest.mode);
    assert_eq!(back.budget_tokens, manifest.budget_tokens);
    assert_eq!(back.used_tokens, manifest.used_tokens);
    assert_eq!(back.total_files, manifest.total_files);
    assert_eq!(back.bundled_files, manifest.bundled_files);
    assert_eq!(back.intelligence_preset, manifest.intelligence_preset);
    assert_eq!(back.artifacts.len(), manifest.artifacts.len());
    assert_eq!(back.included_files.len(), manifest.included_files.len());
    assert_eq!(back.capabilities.len(), manifest.capabilities.len());
}

// =============================================================================
// Scenario: Deterministic HandoffManifest JSON output
// =============================================================================

#[test]
fn given_handoff_manifest_when_serialized_twice_then_output_identical() {
    let manifest = sample_handoff_manifest();
    let json1 = serde_json::to_string_pretty(&manifest).unwrap();
    let json2 = serde_json::to_string_pretty(&manifest).unwrap();
    assert_eq!(json1, json2);
}

// =============================================================================
// Scenario: Token budget partitioning
// =============================================================================

#[test]
fn given_handoff_manifest_then_used_tokens_le_budget_tokens() {
    let manifest = sample_handoff_manifest();
    assert!(manifest.used_tokens <= manifest.budget_tokens);
}

#[test]
fn given_handoff_manifest_then_utilization_pct_consistent() {
    let manifest = sample_handoff_manifest();
    let expected_pct = manifest.used_tokens as f64 / manifest.budget_tokens as f64 * 100.0;
    assert!((manifest.utilization_pct - expected_pct).abs() < 0.1);
}

#[test]
fn given_handoff_manifest_then_bundled_files_le_total_files() {
    let manifest = sample_handoff_manifest();
    assert!(manifest.bundled_files <= manifest.total_files);
}

// =============================================================================
// Scenario: HandoffIntelligence preset handling
// =============================================================================

#[test]
fn given_minimal_intelligence_then_all_optional_fields_none() {
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
}

#[test]
fn given_standard_intelligence_then_tree_and_derived_present() {
    let intel = HandoffIntelligence {
        tree: Some("├── src/\n│   ├── lib.rs\n│   └── main.rs".to_string()),
        tree_depth: Some(3),
        hotspots: Some(vec![HandoffHotspot {
            path: "src/core.rs".to_string(),
            commits: 50,
            lines: 800,
            score: 40_000,
        }]),
        complexity: Some(HandoffComplexity {
            total_functions: 120,
            avg_function_length: 25.5,
            max_function_length: 150,
            avg_cyclomatic: 4.2,
            max_cyclomatic: 32,
            high_risk_files: 3,
        }),
        derived: Some(HandoffDerived {
            total_files: 50,
            total_code: 15_000,
            total_lines: 22_000,
            total_tokens: 60_000,
            lang_count: 3,
            dominant_lang: "Rust".to_string(),
            dominant_pct: 85.5,
        }),
        warnings: vec!["git history limited to 90 days".to_string()],
    };

    assert!(intel.tree.is_some());
    assert_eq!(intel.tree_depth, Some(3));
    assert_eq!(intel.hotspots.as_ref().unwrap().len(), 1);
    assert_eq!(intel.complexity.as_ref().unwrap().total_functions, 120);
    assert_eq!(intel.derived.as_ref().unwrap().dominant_lang, "Rust");
    assert_eq!(intel.warnings.len(), 1);
}

#[test]
fn given_handoff_intelligence_when_roundtripped_then_data_preserved() {
    let intel = HandoffIntelligence {
        tree: Some("root".to_string()),
        tree_depth: Some(2),
        hotspots: Some(vec![]),
        complexity: None,
        derived: Some(HandoffDerived {
            total_files: 10,
            total_code: 500,
            total_lines: 800,
            total_tokens: 2000,
            lang_count: 1,
            dominant_lang: "Python".to_string(),
            dominant_pct: 100.0,
        }),
        warnings: vec![],
    };

    let json_str = serde_json::to_string(&intel).unwrap();
    let back: HandoffIntelligence = serde_json::from_str(&json_str).unwrap();
    assert_eq!(back.tree, intel.tree);
    assert_eq!(back.tree_depth, intel.tree_depth);
    assert_eq!(back.derived.as_ref().unwrap().dominant_lang, "Python");
}

// =============================================================================
// Scenario: HandoffHotspot serialization
// =============================================================================

#[test]
fn given_hotspot_when_serialized_then_all_fields_present() {
    let hotspot = HandoffHotspot {
        path: "src/engine.rs".to_string(),
        commits: 100,
        lines: 1500,
        score: 150_000,
    };
    let json: Value = serde_json::to_value(hotspot).unwrap();
    assert_eq!(json["path"], "src/engine.rs");
    assert_eq!(json["commits"], 100);
    assert_eq!(json["lines"], 1500);
    assert_eq!(json["score"], 150_000);
}

// =============================================================================
// Scenario: HandoffComplexity metrics
// =============================================================================

#[test]
fn given_complexity_report_then_avg_le_max() {
    let complexity = HandoffComplexity {
        total_functions: 80,
        avg_function_length: 30.0,
        max_function_length: 200,
        avg_cyclomatic: 5.5,
        max_cyclomatic: 42,
        high_risk_files: 2,
    };
    assert!(complexity.avg_cyclomatic <= complexity.max_cyclomatic as f64);
    assert!(complexity.avg_function_length <= complexity.max_function_length as f64);
}

// =============================================================================
// Scenario: ContextReceipt construction and serialization
// =============================================================================

#[test]
fn given_context_receipt_then_schema_version_matches_constant() {
    let receipt = sample_context_receipt();
    assert_eq!(receipt.schema_version, CONTEXT_SCHEMA_VERSION);
}

#[test]
fn given_context_receipt_when_serialized_then_json_has_required_fields() {
    let receipt = sample_context_receipt();
    let json: Value = serde_json::to_value(receipt).unwrap();

    assert_eq!(json["schema_version"], CONTEXT_SCHEMA_VERSION);
    assert!(json["generated_at_ms"].is_number());
    assert!(json["tool"].is_object());
    assert_eq!(json["mode"], "context");
    assert!(json["budget_tokens"].is_number());
    assert!(json["used_tokens"].is_number());
    assert!(json["utilization_pct"].is_number());
    assert!(json["strategy"].is_string());
    assert!(json["rank_by"].is_string());
    assert!(json["file_count"].is_number());
    assert!(json["files"].is_array());
}

#[test]
fn given_context_receipt_when_roundtripped_then_data_preserved() {
    let receipt = sample_context_receipt();
    let json_str = serde_json::to_string(&receipt).unwrap();
    let back: ContextReceipt = serde_json::from_str(&json_str).unwrap();

    assert_eq!(back.schema_version, receipt.schema_version);
    assert_eq!(back.mode, receipt.mode);
    assert_eq!(back.budget_tokens, receipt.budget_tokens);
    assert_eq!(back.used_tokens, receipt.used_tokens);
    assert_eq!(back.file_count, receipt.file_count);
    assert_eq!(back.files.len(), receipt.files.len());
}

#[test]
fn given_context_receipt_then_used_tokens_le_budget() {
    let receipt = sample_context_receipt();
    assert!(receipt.used_tokens <= receipt.budget_tokens);
}

#[test]
fn given_context_receipt_when_serialized_twice_then_output_identical() {
    let receipt = sample_context_receipt();
    let json1 = serde_json::to_string_pretty(&receipt).unwrap();
    let json2 = serde_json::to_string_pretty(&receipt).unwrap();
    assert_eq!(json1, json2);
}

// =============================================================================
// Scenario: ContextReceipt with optional fields
// =============================================================================

#[test]
fn given_context_receipt_when_optional_fields_empty_then_omitted() {
    let receipt = sample_context_receipt();
    let json = serde_json::to_string(&receipt).unwrap();

    assert!(!json.contains("\"rank_by_effective\""));
    assert!(!json.contains("\"fallback_reason\""));
    assert!(!json.contains("\"excluded_by_policy\""));
    assert!(!json.contains("\"token_estimation\""));
    assert!(!json.contains("\"bundle_audit\""));
}

#[test]
fn given_context_receipt_with_fallback_then_present_in_json() {
    let mut receipt = sample_context_receipt();
    receipt.rank_by_effective = Some("tokens".to_string());
    receipt.fallback_reason = Some("churn unavailable without git".to_string());

    let json: Value = serde_json::to_value(receipt).unwrap();
    assert_eq!(json["rank_by_effective"], "tokens");
    assert_eq!(json["fallback_reason"], "churn unavailable without git");
}

// =============================================================================
// Scenario: ContextBundleManifest construction and serialization
// =============================================================================

#[test]
fn given_context_bundle_manifest_then_schema_version_matches_constant() {
    let manifest = sample_context_bundle_manifest();
    assert_eq!(manifest.schema_version, CONTEXT_BUNDLE_SCHEMA_VERSION);
}

#[test]
fn given_context_bundle_manifest_when_serialized_then_json_has_required_fields() {
    let manifest = sample_context_bundle_manifest();
    let json: Value = serde_json::to_value(manifest).unwrap();

    assert_eq!(json["schema_version"], CONTEXT_BUNDLE_SCHEMA_VERSION);
    assert!(json["generated_at_ms"].is_number());
    assert!(json["tool"].is_object());
    assert_eq!(json["mode"], "context-bundle");
    assert!(json["budget_tokens"].is_number());
    assert!(json["used_tokens"].is_number());
    assert!(json["utilization_pct"].is_number());
    assert!(json["file_count"].is_number());
    assert!(json["bundle_bytes"].is_number());
    assert!(json["artifacts"].is_array());
    assert!(json["included_files"].is_array());
    assert!(json["excluded_paths"].is_array());
    assert!(json["excluded_patterns"].is_array());
}

#[test]
fn given_context_bundle_manifest_when_roundtripped_then_data_preserved() {
    let manifest = sample_context_bundle_manifest();
    let json_str = serde_json::to_string(&manifest).unwrap();
    let back: ContextBundleManifest = serde_json::from_str(&json_str).unwrap();

    assert_eq!(back.schema_version, manifest.schema_version);
    assert_eq!(back.mode, manifest.mode);
    assert_eq!(back.budget_tokens, manifest.budget_tokens);
    assert_eq!(back.bundle_bytes, manifest.bundle_bytes);
    assert_eq!(back.file_count, manifest.file_count);
    assert_eq!(back.artifacts.len(), manifest.artifacts.len());
}

// =============================================================================
// Scenario: ContextLogRecord
// =============================================================================

#[test]
fn given_context_log_record_when_serialized_then_all_fields_present() {
    let record = ContextLogRecord {
        schema_version: CONTEXT_SCHEMA_VERSION,
        generated_at_ms: 1700000000000,
        tool: sample_tool_info(),
        budget_tokens: 50_000,
        used_tokens: 48_000,
        utilization_pct: 96.0,
        strategy: "greedy".to_string(),
        rank_by: "code".to_string(),
        file_count: 25,
        total_bytes: 200_000,
        output_destination: "stdout".to_string(),
    };

    let json: Value = serde_json::to_value(record).unwrap();
    assert_eq!(json["schema_version"], CONTEXT_SCHEMA_VERSION);
    assert!(json["budget_tokens"].is_number());
    assert_eq!(json["output_destination"], "stdout");
}

#[test]
fn given_context_log_record_when_roundtripped_then_data_preserved() {
    let record = ContextLogRecord {
        schema_version: CONTEXT_SCHEMA_VERSION,
        generated_at_ms: 1700000000000,
        tool: sample_tool_info(),
        budget_tokens: 10_000,
        used_tokens: 9_500,
        utilization_pct: 95.0,
        strategy: "spread".to_string(),
        rank_by: "tokens".to_string(),
        file_count: 8,
        total_bytes: 38_000,
        output_destination: "file.json".to_string(),
    };

    let json_str = serde_json::to_string(&record).unwrap();
    let back: ContextLogRecord = serde_json::from_str(&json_str).unwrap();
    assert_eq!(back.strategy, "spread");
    assert_eq!(back.file_count, 8);
}

// =============================================================================
// Scenario: File selection with InclusionPolicy
// =============================================================================

#[test]
fn given_context_file_row_with_skip_policy_then_effective_tokens_zero() {
    let row = ContextFileRow {
        path: "vendor/huge.min.js".to_string(),
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
        policy_reason: Some("minified vendored file".to_string()),
        classifications: vec![FileClassification::Minified, FileClassification::Vendored],
    };

    assert_eq!(row.effective_tokens, Some(0));
    assert_eq!(row.policy, InclusionPolicy::Skip);
    assert_eq!(row.classifications.len(), 2);
}

#[test]
fn given_context_file_row_with_head_tail_then_effective_tokens_less() {
    let row = ContextFileRow {
        path: "gen/big.rs".to_string(),
        module: "gen".to_string(),
        lang: "Rust".to_string(),
        tokens: 50_000,
        code: 12_000,
        lines: 18_000,
        bytes: 200_000,
        value: 20,
        rank_reason: "code".to_string(),
        policy: InclusionPolicy::HeadTail,
        effective_tokens: Some(5_000),
        policy_reason: Some("exceeds per-file cap".to_string()),
        classifications: vec![FileClassification::Generated],
    };

    assert!(row.effective_tokens.unwrap() < row.tokens);
}

// =============================================================================
// Scenario: HandoffManifest JSON has no null required fields
// =============================================================================

#[test]
fn given_handoff_manifest_then_json_has_no_null_required_fields() {
    let manifest = sample_handoff_manifest();
    let json: Value = serde_json::to_value(manifest).unwrap();
    let obj = json.as_object().unwrap();

    for (key, value) in obj {
        assert!(
            !value.is_null(),
            "HandoffManifest field '{key}' should not be null"
        );
    }
}

// =============================================================================
// Scenario: Snapshot tests (insta)
// =============================================================================

#[test]
fn snapshot_handoff_manifest_minimal() {
    let manifest = sample_handoff_manifest();
    insta::assert_json_snapshot!("handoff_manifest_minimal", manifest);
}

#[test]
fn snapshot_context_receipt_minimal() {
    let receipt = sample_context_receipt();
    insta::assert_json_snapshot!("context_receipt_minimal", receipt);
}

#[test]
fn snapshot_context_bundle_manifest_minimal() {
    let manifest = sample_context_bundle_manifest();
    insta::assert_json_snapshot!("context_bundle_manifest_minimal", manifest);
}

#[test]
fn snapshot_handoff_intelligence_standard() {
    let intel = HandoffIntelligence {
        tree: Some("├── src/\n│   └── lib.rs".to_string()),
        tree_depth: Some(2),
        hotspots: Some(vec![HandoffHotspot {
            path: "src/lib.rs".to_string(),
            commits: 30,
            lines: 500,
            score: 15_000,
        }]),
        complexity: Some(HandoffComplexity {
            total_functions: 40,
            avg_function_length: 20.0,
            max_function_length: 80,
            avg_cyclomatic: 3.5,
            max_cyclomatic: 18,
            high_risk_files: 1,
        }),
        derived: Some(HandoffDerived {
            total_files: 25,
            total_code: 8_000,
            total_lines: 12_000,
            total_tokens: 32_000,
            lang_count: 2,
            dominant_lang: "Rust".to_string(),
            dominant_pct: 90.0,
        }),
        warnings: vec![],
    };
    insta::assert_json_snapshot!("handoff_intelligence_standard", intel);
}

// =============================================================================
// Scenario: Cross-receipt schema version consistency
// =============================================================================

#[test]
fn given_all_receipt_types_then_schema_versions_are_independent() {
    // Each receipt family has its own schema version
    assert_ne!(HANDOFF_SCHEMA_VERSION as u32, CONTEXT_SCHEMA_VERSION);
    assert_ne!(CONTEXT_SCHEMA_VERSION as u32, CONTEXT_BUNDLE_SCHEMA_VERSION);

    let handoff = sample_handoff_manifest();
    let context = sample_context_receipt();
    let bundle = sample_context_bundle_manifest();

    assert_eq!(handoff.schema_version, HANDOFF_SCHEMA_VERSION);
    assert_eq!(context.schema_version, CONTEXT_SCHEMA_VERSION);
    assert_eq!(bundle.schema_version, CONTEXT_BUNDLE_SCHEMA_VERSION);
}

// =============================================================================
// Scenario: HandoffExcludedPath and ContextExcludedPath
// =============================================================================

#[test]
fn given_handoff_excluded_path_when_roundtripped_then_preserved() {
    let path = HandoffExcludedPath {
        path: "target/debug".to_string(),
        reason: "build artifact".to_string(),
    };
    let json_str = serde_json::to_string(&path).unwrap();
    let back: HandoffExcludedPath = serde_json::from_str(&json_str).unwrap();
    assert_eq!(back.path, "target/debug");
    assert_eq!(back.reason, "build artifact");
}

#[test]
fn given_context_excluded_path_when_roundtripped_then_preserved() {
    let path = ContextExcludedPath {
        path: "dist/".to_string(),
        reason: "build output".to_string(),
    };
    let json_str = serde_json::to_string(&path).unwrap();
    let back: ContextExcludedPath = serde_json::from_str(&json_str).unwrap();
    assert_eq!(back.path, "dist/");
}

// =============================================================================
// Scenario: ArtifactEntry with and without hash
// =============================================================================

#[test]
fn given_artifact_with_hash_then_hash_present_in_json() {
    let entry = ArtifactEntry {
        name: "code.txt".to_string(),
        path: "out/code.txt".to_string(),
        description: "Code bundle".to_string(),
        bytes: 50_000,
        hash: Some(ArtifactHash {
            algo: "blake3".to_string(),
            hash: "deadbeef".to_string(),
        }),
    };
    let json: Value = serde_json::to_value(entry).unwrap();
    assert!(json["hash"].is_object());
    assert_eq!(json["hash"]["algo"], "blake3");
}

#[test]
fn given_artifact_without_hash_then_hash_omitted_from_json() {
    let entry = ArtifactEntry {
        name: "map.jsonl".to_string(),
        path: "out/map.jsonl".to_string(),
        description: "File map".to_string(),
        bytes: 1_000,
        hash: None,
    };
    let json = serde_json::to_string(&entry).unwrap();
    assert!(!json.contains("\"hash\""));
}

// =============================================================================
// Scenario: HandoffDerived dominant language percentage
// =============================================================================

#[test]
fn given_handoff_derived_then_dominant_pct_is_between_0_and_100() {
    let derived = HandoffDerived {
        total_files: 100,
        total_code: 50_000,
        total_lines: 75_000,
        total_tokens: 200_000,
        lang_count: 5,
        dominant_lang: "TypeScript".to_string(),
        dominant_pct: 42.5,
    };
    assert!(derived.dominant_pct >= 0.0 && derived.dominant_pct <= 100.0);
}
