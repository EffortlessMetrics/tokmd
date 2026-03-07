//! Schema contract tests for `tokmd-analysis-types` receipt family.
//!
//! These tests verify that analysis receipt schemas are correct, stable,
//! and backwards-compatible.

use serde_json::Value;
use std::collections::BTreeMap;
use tokmd_analysis_types::*;
use tokmd_types::{ScanStatus, ToolInfo};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_analysis_receipt() -> AnalysisReceipt {
    AnalysisReceipt {
        schema_version: ANALYSIS_SCHEMA_VERSION,
        generated_at_ms: 1_700_000_000_000,
        tool: ToolInfo::current(),
        mode: "analyze".into(),
        status: ScanStatus::Complete,
        warnings: vec![],
        source: AnalysisSource {
            inputs: vec![".".into()],
            export_path: None,
            base_receipt_path: None,
            export_schema_version: None,
            export_generated_at_ms: None,
            base_signature: None,
            module_roots: vec![],
            module_depth: 1,
            children: "collapse".into(),
        },
        args: AnalysisArgsMeta {
            preset: "receipt".into(),
            format: "json".into(),
            window_tokens: None,
            git: None,
            max_files: None,
            max_bytes: None,
            max_commits: None,
            max_commit_files: None,
            max_file_bytes: None,
            import_granularity: "module".into(),
            effort_model: None,
            effort_layer: None,
            base_ref: None,
            head_ref: None,
            monte_carlo: None,
            mc_iterations: None,
            mc_seed: None,
        },
        archetype: None,
        topics: None,
        entropy: None,
        predictive_churn: None,
        corporate_fingerprint: None,
        license: None,
        derived: None,
        assets: None,
        deps: None,
        git: None,
        imports: None,
        dup: None,
        complexity: None,
        api_surface: None,
        fun: None,
        effort: None,
    }
}

// ===========================================================================
// 1. Schema version constant
// ===========================================================================

#[test]
fn analysis_schema_version_is_positive() {
    #[allow(clippy::assertions_on_constants)]
    {
        assert!(ANALYSIS_SCHEMA_VERSION > 0);
    }
}

#[test]
fn analysis_schema_version_value() {
    assert_eq!(ANALYSIS_SCHEMA_VERSION, 9);
}

#[test]
fn baseline_version_is_positive() {
    #[allow(clippy::assertions_on_constants)]
    {
        assert!(BASELINE_VERSION > 0);
    }
}

// ===========================================================================
// 2. JSON roundtrip for AnalysisReceipt
// ===========================================================================

#[test]
fn analysis_receipt_json_roundtrip() {
    let receipt = make_analysis_receipt();
    let json = serde_json::to_string_pretty(&receipt).unwrap();
    let back: AnalysisReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema_version, ANALYSIS_SCHEMA_VERSION);
    assert_eq!(back.mode, "analyze");
}

#[test]
fn analysis_receipt_envelope_has_schema_version() {
    let receipt = make_analysis_receipt();
    let json = serde_json::to_string(&receipt).unwrap();
    let val: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(val["schema_version"], ANALYSIS_SCHEMA_VERSION);
}

#[test]
fn analysis_receipt_with_all_optional_sections_roundtrip() {
    let mut receipt = make_analysis_receipt();
    receipt.archetype = Some(Archetype {
        kind: "library".into(),
        evidence: vec!["Cargo.toml".into()],
    });
    receipt.entropy = Some(EntropyReport {
        suspects: vec![EntropyFinding {
            path: "secrets.bin".into(),
            module: "src".into(),
            entropy_bits_per_byte: 7.8,
            sample_bytes: 1024,
            class: EntropyClass::High,
        }],
    });
    receipt.fun = Some(FunReport {
        eco_label: Some(EcoLabel {
            score: 0.9,
            label: "Green".into(),
            bytes: 1000,
            notes: "Eco-friendly".into(),
        }),
    });
    let json = serde_json::to_string_pretty(&receipt).unwrap();
    let back: AnalysisReceipt = serde_json::from_str(&json).unwrap();
    assert!(back.archetype.is_some());
    assert!(back.entropy.is_some());
    assert!(back.fun.is_some());
}

// ===========================================================================
// 3. Optional fields don't break deserialization of old format
// ===========================================================================

#[test]
fn analysis_receipt_ignores_extra_fields() {
    let receipt = make_analysis_receipt();
    let mut json: Value = serde_json::to_value(&receipt).unwrap();
    json["future_enricher"] = Value::String("new_data".into());
    json["v9_section"] = serde_json::json!({"score": 42});
    let back: AnalysisReceipt = serde_json::from_value(json).unwrap();
    assert_eq!(back.schema_version, ANALYSIS_SCHEMA_VERSION);
}

#[test]
fn analysis_receipt_missing_optional_sections_ok() {
    let receipt = make_analysis_receipt();
    let mut json: Value = serde_json::to_value(&receipt).unwrap();
    let obj = json.as_object_mut().unwrap();
    obj.remove("api_surface");
    obj.remove("fun");
    obj.remove("complexity");
    let back: AnalysisReceipt = serde_json::from_value(json).unwrap();
    assert!(back.api_surface.is_none());
    assert!(back.fun.is_none());
    assert!(back.complexity.is_none());
}

// ===========================================================================
// 4. Enum variants serialize to expected case
// ===========================================================================

#[test]
fn entropy_class_serializes_snake_case() {
    assert_eq!(
        serde_json::to_string(&EntropyClass::Low).unwrap(),
        "\"low\""
    );
    assert_eq!(
        serde_json::to_string(&EntropyClass::Normal).unwrap(),
        "\"normal\""
    );
    assert_eq!(
        serde_json::to_string(&EntropyClass::Suspicious).unwrap(),
        "\"suspicious\""
    );
    assert_eq!(
        serde_json::to_string(&EntropyClass::High).unwrap(),
        "\"high\""
    );
}

#[test]
fn trend_class_serializes_snake_case() {
    assert_eq!(
        serde_json::to_string(&TrendClass::Rising).unwrap(),
        "\"rising\""
    );
    assert_eq!(
        serde_json::to_string(&TrendClass::Flat).unwrap(),
        "\"flat\""
    );
    assert_eq!(
        serde_json::to_string(&TrendClass::Falling).unwrap(),
        "\"falling\""
    );
}

#[test]
fn license_source_kind_serializes_snake_case() {
    assert_eq!(
        serde_json::to_string(&LicenseSourceKind::Metadata).unwrap(),
        "\"metadata\""
    );
    assert_eq!(
        serde_json::to_string(&LicenseSourceKind::Text).unwrap(),
        "\"text\""
    );
}

#[test]
fn complexity_risk_serializes_snake_case() {
    assert_eq!(
        serde_json::to_string(&ComplexityRisk::Low).unwrap(),
        "\"low\""
    );
    assert_eq!(
        serde_json::to_string(&ComplexityRisk::Moderate).unwrap(),
        "\"moderate\""
    );
    assert_eq!(
        serde_json::to_string(&ComplexityRisk::Critical).unwrap(),
        "\"critical\""
    );
}

#[test]
fn technical_debt_level_serializes_snake_case() {
    assert_eq!(
        serde_json::to_string(&TechnicalDebtLevel::Low).unwrap(),
        "\"low\""
    );
    assert_eq!(
        serde_json::to_string(&TechnicalDebtLevel::Moderate).unwrap(),
        "\"moderate\""
    );
    assert_eq!(
        serde_json::to_string(&TechnicalDebtLevel::High).unwrap(),
        "\"high\""
    );
    assert_eq!(
        serde_json::to_string(&TechnicalDebtLevel::Critical).unwrap(),
        "\"critical\""
    );
}

#[test]
fn near_dup_scope_serializes_kebab_case() {
    assert_eq!(
        serde_json::to_string(&NearDupScope::Module).unwrap(),
        "\"module\""
    );
    assert_eq!(
        serde_json::to_string(&NearDupScope::Lang).unwrap(),
        "\"lang\""
    );
    assert_eq!(
        serde_json::to_string(&NearDupScope::Global).unwrap(),
        "\"global\""
    );
}

// ===========================================================================
// 5. Nested type roundtrips
// ===========================================================================

#[test]
fn complexity_baseline_json_roundtrip() {
    let baseline = ComplexityBaseline::new();
    let json = serde_json::to_string_pretty(&baseline).unwrap();
    let back: ComplexityBaseline = serde_json::from_str(&json).unwrap();
    assert_eq!(back.baseline_version, BASELINE_VERSION);
}

#[test]
fn topic_clouds_json_roundtrip() {
    let clouds = TopicClouds {
        per_module: {
            let mut m = BTreeMap::new();
            m.insert(
                "src".into(),
                vec![TopicTerm {
                    term: "parser".into(),
                    score: 0.85,
                    tf: 10,
                    df: 2,
                }],
            );
            m
        },
        overall: vec![TopicTerm {
            term: "analysis".into(),
            score: 0.9,
            tf: 20,
            df: 5,
        }],
    };
    let json = serde_json::to_string_pretty(&clouds).unwrap();
    let back: TopicClouds = serde_json::from_str(&json).unwrap();
    assert_eq!(back.overall.len(), 1);
    assert_eq!(back.per_module.len(), 1);
}

#[test]
fn near_duplicate_report_json_roundtrip() {
    let report = NearDuplicateReport {
        params: NearDupParams {
            scope: NearDupScope::Module,
            threshold: 0.8,
            max_files: 500,
            max_pairs: Some(1000),
            max_file_bytes: None,
            selection_method: None,
            algorithm: None,
            exclude_patterns: vec![],
        },
        pairs: vec![NearDupPairRow {
            left: "a.rs".into(),
            right: "b.rs".into(),
            similarity: 0.92,
            shared_fingerprints: 100,
            left_fingerprints: 110,
            right_fingerprints: 105,
        }],
        files_analyzed: 50,
        files_skipped: 2,
        eligible_files: Some(52),
        clusters: None,
        truncated: false,
        excluded_by_pattern: None,
        stats: None,
    };
    let json = serde_json::to_string_pretty(&report).unwrap();
    let back: NearDuplicateReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.pairs.len(), 1);
    assert!(!back.truncated);
}

// ===========================================================================
// 6. Field name stability
// ===========================================================================

#[test]
fn analysis_receipt_required_fields_present() {
    let receipt = make_analysis_receipt();
    let json = serde_json::to_string(&receipt).unwrap();
    let val: Value = serde_json::from_str(&json).unwrap();
    let obj = val.as_object().unwrap();
    let expected_keys = [
        "schema_version",
        "generated_at_ms",
        "tool",
        "mode",
        "status",
        "warnings",
        "source",
        "args",
    ];
    for key in &expected_keys {
        assert!(obj.contains_key(*key), "Missing expected key: {key}");
    }
}

#[test]
fn analysis_args_meta_field_names_stable() {
    let args = AnalysisArgsMeta {
        preset: "receipt".into(),
        format: "json".into(),
        window_tokens: Some(128_000),
        git: Some(true),
        max_files: None,
        max_bytes: None,
        max_commits: None,
        max_commit_files: None,
        max_file_bytes: None,
        import_granularity: "module".into(),
        effort_model: None,
        effort_layer: None,
        base_ref: None,
        head_ref: None,
        monte_carlo: None,
        mc_iterations: None,
        mc_seed: None,
    };
    let json = serde_json::to_string(&args).unwrap();
    let val: Value = serde_json::from_str(&json).unwrap();
    let obj = val.as_object().unwrap();
    assert!(obj.contains_key("preset"));
    assert!(obj.contains_key("format"));
    assert!(obj.contains_key("import_granularity"));
}

#[test]
fn commit_intent_kind_all_variants_roundtrip() {
    let variants = [
        CommitIntentKind::Feat,
        CommitIntentKind::Fix,
        CommitIntentKind::Refactor,
        CommitIntentKind::Docs,
        CommitIntentKind::Test,
        CommitIntentKind::Chore,
        CommitIntentKind::Ci,
        CommitIntentKind::Build,
        CommitIntentKind::Perf,
        CommitIntentKind::Style,
        CommitIntentKind::Revert,
        CommitIntentKind::Other,
    ];
    for variant in variants {
        let json = serde_json::to_string(&variant).unwrap();
        let back: CommitIntentKind = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}
