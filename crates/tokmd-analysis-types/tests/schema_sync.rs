//! Schema version synchronization tests for analysis receipts.
//!
//! Verify that ANALYSIS_SCHEMA_VERSION is valid and that
//! AnalysisReceipt serializes with the expected fields.

use tokmd_analysis_types::{
    ANALYSIS_SCHEMA_VERSION, AnalysisArgsMeta, AnalysisReceipt, AnalysisSource,
};
use tokmd_types::{ScanStatus, ToolInfo};

// =============================================================================
// Schema version constant is valid
// =============================================================================

#[test]
fn schema_analysis_version_is_positive() {
    assert!(
        ANALYSIS_SCHEMA_VERSION > 0,
        "ANALYSIS_SCHEMA_VERSION must be positive"
    );
}

#[test]
fn schema_analysis_version_matches_expected() {
    assert_eq!(
        ANALYSIS_SCHEMA_VERSION, 8,
        "ANALYSIS_SCHEMA_VERSION must match documented value"
    );
}

// =============================================================================
// Helpers
// =============================================================================

fn base_receipt() -> AnalysisReceipt {
    AnalysisReceipt {
        schema_version: ANALYSIS_SCHEMA_VERSION,
        generated_at_ms: 0,
        tool: ToolInfo {
            name: "tokmd".to_string(),
            version: "0.0.0-test".to_string(),
        },
        mode: "analysis".to_string(),
        status: ScanStatus::Complete,
        warnings: Vec::new(),
        source: AnalysisSource {
            inputs: vec![".".to_string()],
            export_path: None,
            base_receipt_path: None,
            export_schema_version: None,
            export_generated_at_ms: None,
            base_signature: None,
            module_roots: vec!["src".to_string()],
            module_depth: 1,
            children: "separate".to_string(),
        },
        args: AnalysisArgsMeta {
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
    }
}

// =============================================================================
// AnalysisReceipt serialization includes schema_version
// =============================================================================

#[test]
fn schema_analysis_receipt_json_includes_schema_version() {
    let receipt = base_receipt();
    let json = serde_json::to_value(&receipt).expect("AnalysisReceipt must serialize");

    assert_eq!(
        json["schema_version"], ANALYSIS_SCHEMA_VERSION,
        "schema_version must match ANALYSIS_SCHEMA_VERSION"
    );
}

// =============================================================================
// AnalysisReceipt has all required envelope fields
// =============================================================================

#[test]
fn schema_analysis_receipt_has_required_fields() {
    let receipt = base_receipt();
    let json = serde_json::to_value(&receipt).expect("AnalysisReceipt must serialize");

    let required = [
        "schema_version",
        "generated_at_ms",
        "tool",
        "mode",
        "status",
        "warnings",
        "source",
        "args",
    ];
    for field in &required {
        assert!(
            !json[field].is_null(),
            "AnalysisReceipt missing required field: {field}"
        );
    }
}

// =============================================================================
// AnalysisReceipt optional sections serialize as null when absent
// =============================================================================

#[test]
fn schema_analysis_receipt_optional_sections_are_null_when_absent() {
    let receipt = base_receipt();
    let json = serde_json::to_value(&receipt).expect("AnalysisReceipt must serialize");

    let optional_sections = [
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
    for section in &optional_sections {
        assert!(
            json[section].is_null(),
            "Empty {section} should serialize as null"
        );
    }
}

// =============================================================================
// AnalysisReceipt roundtrip preserves schema_version
// =============================================================================

#[test]
fn schema_analysis_receipt_roundtrip_preserves_version() {
    let receipt = base_receipt();
    let json_str = serde_json::to_string(&receipt).expect("serialize");
    let roundtrip: AnalysisReceipt = serde_json::from_str(&json_str).expect("deserialize");
    assert_eq!(roundtrip.schema_version, ANALYSIS_SCHEMA_VERSION);
}
