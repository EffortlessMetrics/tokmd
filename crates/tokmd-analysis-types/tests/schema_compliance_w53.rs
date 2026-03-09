//! Schema compliance tests for analysis receipt types.
//!
//! These tests verify that analysis receipt structures conform to their documented
//! schemas and that schema version constants are correctly maintained.

use serde_json::Value;
use tokmd_analysis_types::{
    AnalysisArgsMeta, AnalysisReceipt, AnalysisSource, Archetype, ComplexityRisk, EntropyReport,
    FunReport, TopicClouds, ANALYSIS_SCHEMA_VERSION,
};
use tokmd_types::{ScanStatus, ToolInfo};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn sample_analysis_receipt() -> AnalysisReceipt {
    AnalysisReceipt {
        schema_version: ANALYSIS_SCHEMA_VERSION,
        generated_at_ms: 1_700_000_000_000,
        tool: ToolInfo {
            name: "tokmd".into(),
            version: "0.0.0-test".into(),
        },
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
            children: "separate".into(),
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
        effort: None,
        complexity: None,
        api_surface: None,
        fun: None,
    }
}

// ---------------------------------------------------------------------------
// 1. Schema version constant
// ---------------------------------------------------------------------------

#[test]
fn analysis_schema_version_is_positive() {
    const _: () = assert!(ANALYSIS_SCHEMA_VERSION > 0);
}

#[test]
fn analysis_schema_version_matches_documented_value() {
    assert_eq!(ANALYSIS_SCHEMA_VERSION, 9);
}

// ---------------------------------------------------------------------------
// 2. AnalysisReceipt JSON has required fields
// ---------------------------------------------------------------------------

#[test]
fn analysis_receipt_json_has_required_fields() {
    let receipt = sample_analysis_receipt();
    let json: Value = serde_json::to_value(&receipt).unwrap();
    let obj = json.as_object().unwrap();

    assert!(obj.contains_key("schema_version"));
    assert!(obj.contains_key("generated_at_ms"));
    assert!(obj.contains_key("tool"));
    assert!(obj.contains_key("mode"));
    assert!(obj.contains_key("status"));
    assert!(obj.contains_key("warnings"));
    assert!(obj.contains_key("source"));
    assert!(obj.contains_key("args"));
}

#[test]
fn analysis_receipt_schema_version_in_json() {
    let receipt = sample_analysis_receipt();
    let json: Value = serde_json::to_value(&receipt).unwrap();
    assert_eq!(json["schema_version"], ANALYSIS_SCHEMA_VERSION);
}

// ---------------------------------------------------------------------------
// 3. Missing enrichments serialize as null
// ---------------------------------------------------------------------------

#[test]
fn missing_enrichments_serialize_as_null() {
    let receipt = sample_analysis_receipt();
    let json: Value = serde_json::to_value(&receipt).unwrap();

    assert!(json["archetype"].is_null());
    assert!(json["topics"].is_null());
    assert!(json["entropy"].is_null());
    assert!(json["derived"].is_null());
    assert!(json["git"].is_null());
    assert!(json["complexity"].is_null());
    assert!(json["fun"].is_null());
}

// ---------------------------------------------------------------------------
// 4. Enrichment types serialize correctly
// ---------------------------------------------------------------------------

#[test]
fn archetype_serializes_correctly() {
    let arch = Archetype {
        kind: "web-app".into(),
        evidence: vec!["package.json".into(), "index.html".into()],
    };
    let json: Value = serde_json::to_value(&arch).unwrap();
    assert_eq!(json["kind"], "web-app");
    assert!(json["evidence"].is_array());
    assert_eq!(json["evidence"].as_array().unwrap().len(), 2);
}

#[test]
fn entropy_report_serializes_correctly() {
    let report = EntropyReport { suspects: vec![] };
    let json: Value = serde_json::to_value(&report).unwrap();
    assert!(json["suspects"].is_array());
}

#[test]
fn fun_report_serializes_correctly() {
    let fun = FunReport { eco_label: None };
    let json: Value = serde_json::to_value(&fun).unwrap();
    assert!(json["eco_label"].is_null());
}

#[test]
fn topic_clouds_serializes_correctly() {
    let topics = TopicClouds {
        per_module: std::collections::BTreeMap::new(),
        overall: vec![],
    };
    let json: Value = serde_json::to_value(&topics).unwrap();
    assert!(json["per_module"].is_object());
    assert!(json["overall"].is_array());
}

// ---------------------------------------------------------------------------
// 5. Serde roundtrip
// ---------------------------------------------------------------------------

#[test]
fn analysis_receipt_roundtrip() {
    let receipt = sample_analysis_receipt();
    let json = serde_json::to_string(&receipt).unwrap();
    let back: AnalysisReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema_version, ANALYSIS_SCHEMA_VERSION);
    assert_eq!(back.mode, "analyze");
    assert!(back.archetype.is_none());
    assert!(back.derived.is_none());
}

#[test]
fn analysis_receipt_with_enrichments_roundtrip() {
    let mut receipt = sample_analysis_receipt();
    receipt.archetype = Some(Archetype {
        kind: "cli-tool".into(),
        evidence: vec!["main.rs".into()],
    });
    receipt.fun = Some(FunReport { eco_label: None });

    let json = serde_json::to_string(&receipt).unwrap();
    let back: AnalysisReceipt = serde_json::from_str(&json).unwrap();
    assert!(back.archetype.is_some());
    assert_eq!(back.archetype.unwrap().kind, "cli-tool");
    assert!(back.fun.is_some());
}

// ---------------------------------------------------------------------------
// 6. Preset metadata is included
// ---------------------------------------------------------------------------

#[test]
fn preset_metadata_in_args() {
    let receipt = sample_analysis_receipt();
    let json: Value = serde_json::to_value(&receipt).unwrap();

    assert_eq!(json["args"]["preset"], "receipt");
    assert_eq!(json["args"]["format"], "json");
    assert_eq!(json["args"]["import_granularity"], "module");
}

// ---------------------------------------------------------------------------
// 7. Complexity risk enum roundtrip
// ---------------------------------------------------------------------------

#[test]
fn complexity_risk_serde_roundtrip() {
    for variant in [
        ComplexityRisk::Low,
        ComplexityRisk::Moderate,
        ComplexityRisk::High,
        ComplexityRisk::Critical,
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: ComplexityRisk = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

// ---------------------------------------------------------------------------
// 8. Source structure is correct
// ---------------------------------------------------------------------------

#[test]
fn analysis_source_structure() {
    let receipt = sample_analysis_receipt();
    let json: Value = serde_json::to_value(&receipt).unwrap();
    let source = &json["source"];

    assert!(source["inputs"].is_array());
    assert!(source.get("module_roots").unwrap().is_array());
    assert!(source["module_depth"].is_number());
}
