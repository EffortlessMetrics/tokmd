//! Schema validation tests for tokmd-analysis-types receipt types.
//!
//! These tests verify that AnalysisReceipt JSON output matches expected
//! structure, required fields are present, schema versions are correct,
//! and round-trip serialization preserves data.

use serde_json::Value;
use tokmd_analysis_types::{
    ANALYSIS_SCHEMA_VERSION, AnalysisArgsMeta, AnalysisReceipt, AnalysisSource, BASELINE_VERSION,
};
use tokmd_types::{ScanStatus, ToolInfo};

// =============================================================================
// Helpers
// =============================================================================

fn sample_tool_info() -> ToolInfo {
    ToolInfo {
        name: "tokmd".to_string(),
        version: "0.0.0-test".to_string(),
    }
}

fn sample_analysis_source() -> AnalysisSource {
    AnalysisSource {
        inputs: vec![".".to_string()],
        export_path: None,
        base_receipt_path: None,
        export_schema_version: None,
        export_generated_at_ms: None,
        base_signature: None,
        module_roots: vec![],
        module_depth: 1,
        children: "separate".to_string(),
    }
}

fn sample_analysis_args() -> AnalysisArgsMeta {
    AnalysisArgsMeta {
        preset: "receipt".to_string(),
        format: "json".to_string(),
        window_tokens: None,
        git: None,
        max_files: None,
        max_bytes: None,
        max_commits: None,
        max_commit_files: None,
        max_file_bytes: None,
        import_granularity: "module".to_string(),
    }
}

fn sample_analysis_receipt() -> AnalysisReceipt {
    AnalysisReceipt {
        schema_version: ANALYSIS_SCHEMA_VERSION,
        generated_at_ms: 1700000000000,
        tool: sample_tool_info(),
        mode: "analyze".to_string(),
        status: ScanStatus::Complete,
        warnings: vec![],
        source: sample_analysis_source(),
        args: sample_analysis_args(),
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
    }
}

// =============================================================================
// Schema version constants
// =============================================================================

#[test]
fn analysis_schema_version_matches_expected() {
    assert_eq!(ANALYSIS_SCHEMA_VERSION, 8, "ANALYSIS_SCHEMA_VERSION changed — update docs/SCHEMA.md and docs/schema.json");
}

#[test]
fn baseline_version_matches_expected() {
    assert_eq!(BASELINE_VERSION, 1, "BASELINE_VERSION changed — update docs/SCHEMA.md");
}

// =============================================================================
// AnalysisReceipt envelope validation
// =============================================================================

#[test]
fn analysis_receipt_json_contains_required_envelope_fields() {
    let receipt = sample_analysis_receipt();
    let json: Value = serde_json::to_value(&receipt).unwrap();

    assert_eq!(json["schema_version"], ANALYSIS_SCHEMA_VERSION);
    assert!(json["generated_at_ms"].is_number());
    assert_eq!(json["tool"]["name"], "tokmd");
    assert!(json["tool"]["version"].is_string());
    assert_eq!(json["mode"], "analyze");
    assert!(json["status"].is_string());
    assert!(json["warnings"].is_array());
}

#[test]
fn analysis_receipt_json_contains_source_fields() {
    let receipt = sample_analysis_receipt();
    let json: Value = serde_json::to_value(&receipt).unwrap();
    let source = &json["source"];

    assert!(source["inputs"].is_array());
    assert!(source["module_roots"].is_array());
    assert!(source["module_depth"].is_number());
    assert!(source["children"].is_string());
}

#[test]
fn analysis_receipt_json_contains_args_fields() {
    let receipt = sample_analysis_receipt();
    let json: Value = serde_json::to_value(&receipt).unwrap();
    let args = &json["args"];

    assert_eq!(args["preset"], "receipt");
    assert_eq!(args["format"], "json");
    assert!(args["import_granularity"].is_string());
}

// =============================================================================
// AnalysisReceipt roundtrip
// =============================================================================

#[test]
fn analysis_receipt_roundtrip() {
    let receipt = sample_analysis_receipt();
    let json_str = serde_json::to_string(&receipt).unwrap();
    let deserialized: AnalysisReceipt = serde_json::from_str(&json_str).unwrap();

    assert_eq!(deserialized.schema_version, receipt.schema_version);
    assert_eq!(deserialized.generated_at_ms, receipt.generated_at_ms);
    assert_eq!(deserialized.mode, receipt.mode);
    assert_eq!(deserialized.tool.name, receipt.tool.name);
    assert_eq!(deserialized.tool.version, receipt.tool.version);
    assert_eq!(deserialized.args.preset, receipt.args.preset);
    assert_eq!(deserialized.source.inputs, receipt.source.inputs);
}

#[test]
fn analysis_receipt_roundtrip_preserves_null_optional_fields() {
    let receipt = sample_analysis_receipt();
    let json_str = serde_json::to_string(&receipt).unwrap();
    let deserialized: AnalysisReceipt = serde_json::from_str(&json_str).unwrap();

    assert!(deserialized.archetype.is_none());
    assert!(deserialized.topics.is_none());
    assert!(deserialized.entropy.is_none());
    assert!(deserialized.predictive_churn.is_none());
    assert!(deserialized.corporate_fingerprint.is_none());
    assert!(deserialized.license.is_none());
    assert!(deserialized.derived.is_none());
    assert!(deserialized.assets.is_none());
    assert!(deserialized.deps.is_none());
    assert!(deserialized.git.is_none());
    assert!(deserialized.imports.is_none());
    assert!(deserialized.dup.is_none());
    assert!(deserialized.complexity.is_none());
    assert!(deserialized.api_surface.is_none());
    assert!(deserialized.fun.is_none());
}

// =============================================================================
// Optional analysis sections as null in JSON
// =============================================================================

#[test]
fn analysis_receipt_optional_sections_serialize_as_null() {
    let receipt = sample_analysis_receipt();
    let json: Value = serde_json::to_value(&receipt).unwrap();

    let optional_fields = [
        "archetype",
        "topics",
        "entropy",
        "predictive_churn",
        "corporate_fingerprint",
        "license",
        "derived",
        "assets",
        "deps",
        "git",
        "imports",
        "dup",
        "complexity",
        "api_surface",
        "fun",
    ];

    for field in &optional_fields {
        assert!(
            json[field].is_null(),
            "Optional field '{field}' should be null when None"
        );
    }
}

// =============================================================================
// AnalysisSource validation
// =============================================================================

#[test]
fn analysis_source_roundtrip() {
    let source = sample_analysis_source();
    let json_str = serde_json::to_string(&source).unwrap();
    let deserialized: AnalysisSource = serde_json::from_str(&json_str).unwrap();

    assert_eq!(deserialized.inputs, source.inputs);
    assert_eq!(deserialized.module_roots, source.module_roots);
    assert_eq!(deserialized.module_depth, source.module_depth);
    assert_eq!(deserialized.children, source.children);
}

#[test]
fn analysis_source_optional_fields_absent_when_none() {
    let source = sample_analysis_source();
    let json: Value = serde_json::to_value(&source).unwrap();

    assert!(json["export_path"].is_null());
    assert!(json["base_receipt_path"].is_null());
    assert!(json["export_schema_version"].is_null());
    assert!(json["export_generated_at_ms"].is_null());
    assert!(json["base_signature"].is_null());
}

// =============================================================================
// AnalysisArgsMeta validation
// =============================================================================

#[test]
fn analysis_args_meta_roundtrip() {
    let args = sample_analysis_args();
    let json_str = serde_json::to_string(&args).unwrap();
    let deserialized: AnalysisArgsMeta = serde_json::from_str(&json_str).unwrap();

    assert_eq!(deserialized.preset, args.preset);
    assert_eq!(deserialized.format, args.format);
    assert_eq!(deserialized.import_granularity, args.import_granularity);
}

#[test]
fn analysis_args_meta_optional_fields_null_when_none() {
    let args = sample_analysis_args();
    let json: Value = serde_json::to_value(&args).unwrap();

    assert!(json["window_tokens"].is_null());
    assert!(json["git"].is_null());
    assert!(json["max_files"].is_null());
    assert!(json["max_bytes"].is_null());
    assert!(json["max_commits"].is_null());
    assert!(json["max_commit_files"].is_null());
    assert!(json["max_file_bytes"].is_null());
}

// =============================================================================
// JSON shape stability: no unexpected keys
// =============================================================================

#[test]
fn analysis_receipt_top_level_keys_are_known() {
    let receipt = sample_analysis_receipt();
    let json: Value = serde_json::to_value(&receipt).unwrap();
    let obj = json.as_object().unwrap();

    let known_keys: Vec<&str> = vec![
        "schema_version",
        "generated_at_ms",
        "tool",
        "mode",
        "status",
        "warnings",
        "source",
        "args",
        "archetype",
        "topics",
        "entropy",
        "predictive_churn",
        "corporate_fingerprint",
        "license",
        "derived",
        "assets",
        "deps",
        "git",
        "imports",
        "dup",
        "complexity",
        "api_surface",
        "fun",
    ];

    for key in obj.keys() {
        assert!(
            known_keys.contains(&key.as_str()),
            "Unexpected top-level key in AnalysisReceipt: '{key}'"
        );
    }

    for expected in &known_keys {
        assert!(
            obj.contains_key(*expected),
            "Missing expected key in AnalysisReceipt: '{expected}'"
        );
    }
}

// =============================================================================
// Enum serialization for analysis-adjacent types
// =============================================================================

#[test]
fn scan_status_values_in_analysis_context() {
    let complete = serde_json::to_string(&ScanStatus::Complete).unwrap();
    let partial = serde_json::to_string(&ScanStatus::Partial).unwrap();

    assert_eq!(complete, "\"complete\"");
    assert_eq!(partial, "\"partial\"");
}

// =============================================================================
// Schema version embedded in receipt
// =============================================================================

#[test]
fn analysis_receipt_schema_version_field_matches_constant() {
    let receipt = sample_analysis_receipt();

    assert_eq!(receipt.schema_version, ANALYSIS_SCHEMA_VERSION);

    let json: Value = serde_json::to_value(&receipt).unwrap();
    assert_eq!(
        json["schema_version"].as_u64().unwrap(),
        ANALYSIS_SCHEMA_VERSION as u64
    );
}
