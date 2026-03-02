//! Schema validation tests for tokmd-types receipt types.
//!
//! These tests verify that JSON output matches expected structure,
//! required fields are present, schema versions are correct,
//! and round-trip serialization preserves data.

use serde_json::Value;
use tokmd_types::{
    CONTEXT_BUNDLE_SCHEMA_VERSION, CONTEXT_SCHEMA_VERSION, ChildIncludeMode, ChildrenMode,
    ConfigMode, DiffReceipt, DiffTotals, ExportArgsMeta, ExportData, ExportFormat, ExportReceipt,
    HANDOFF_SCHEMA_VERSION, LangArgsMeta, LangReceipt, LangReport, LangRow, ModuleArgsMeta,
    ModuleReceipt, ModuleReport, ModuleRow, RedactMode, SCHEMA_VERSION, ScanArgs, ScanStatus,
    ToolInfo, Totals, cockpit::COCKPIT_SCHEMA_VERSION,
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

fn sample_scan_args() -> ScanArgs {
    ScanArgs {
        paths: vec![".".to_string()],
        excluded: vec![],
        excluded_redacted: false,
        config: ConfigMode::Auto,
        hidden: false,
        no_ignore: false,
        no_ignore_parent: false,
        no_ignore_dot: false,
        no_ignore_vcs: false,
        treat_doc_strings_as_comments: false,
    }
}

fn sample_totals() -> Totals {
    Totals {
        code: 1000,
        lines: 1500,
        files: 10,
        bytes: 50000,
        tokens: 12500,
        avg_lines: 150,
    }
}

fn sample_lang_receipt() -> LangReceipt {
    LangReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 1700000000000,
        tool: sample_tool_info(),
        mode: "lang".to_string(),
        status: ScanStatus::Complete,
        warnings: vec![],
        scan: sample_scan_args(),
        args: LangArgsMeta {
            format: "json".to_string(),
            top: 0,
            with_files: false,
            children: ChildrenMode::Collapse,
        },
        report: LangReport {
            rows: vec![LangRow {
                lang: "Rust".to_string(),
                code: 1000,
                lines: 1500,
                files: 10,
                bytes: 50000,
                tokens: 12500,
                avg_lines: 150,
            }],
            total: sample_totals(),
            with_files: false,
            children: ChildrenMode::Collapse,
            top: 0,
        },
    }
}

fn sample_module_receipt() -> ModuleReceipt {
    ModuleReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 1700000000000,
        tool: sample_tool_info(),
        mode: "module".to_string(),
        status: ScanStatus::Complete,
        warnings: vec![],
        scan: sample_scan_args(),
        args: ModuleArgsMeta {
            format: "json".to_string(),
            module_roots: vec![],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
            top: 0,
        },
        report: ModuleReport {
            rows: vec![ModuleRow {
                module: "src".to_string(),
                code: 1000,
                lines: 1500,
                files: 10,
                bytes: 50000,
                tokens: 12500,
                avg_lines: 150,
            }],
            total: sample_totals(),
            module_roots: vec![],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
            top: 0,
        },
    }
}

fn sample_export_receipt() -> ExportReceipt {
    ExportReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 1700000000000,
        tool: sample_tool_info(),
        mode: "export".to_string(),
        status: ScanStatus::Complete,
        warnings: vec![],
        scan: sample_scan_args(),
        args: ExportArgsMeta {
            format: ExportFormat::Json,
            module_roots: vec![],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
            min_code: 0,
            max_rows: 10000,
            redact: RedactMode::None,
            strip_prefix: None,
            strip_prefix_redacted: false,
        },
        data: ExportData {
            rows: vec![],
            module_roots: vec![],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
        },
    }
}

fn sample_diff_receipt() -> DiffReceipt {
    DiffReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 1700000000000,
        tool: sample_tool_info(),
        mode: "diff".to_string(),
        from_source: "v1.0.0".to_string(),
        to_source: "v2.0.0".to_string(),
        diff_rows: vec![],
        totals: DiffTotals::default(),
    }
}

// =============================================================================
// Schema version constants
// =============================================================================

#[test]
fn schema_version_constants_match_expected_values() {
    assert_eq!(SCHEMA_VERSION, 2, "SCHEMA_VERSION changed — update docs/SCHEMA.md and docs/schema.json");
    assert_eq!(COCKPIT_SCHEMA_VERSION, 3, "COCKPIT_SCHEMA_VERSION changed — update docs/SCHEMA.md and docs/schema.json");
    assert_eq!(HANDOFF_SCHEMA_VERSION, 5, "HANDOFF_SCHEMA_VERSION changed — update docs/SCHEMA.md and docs/schema.json");
    assert_eq!(CONTEXT_SCHEMA_VERSION, 4, "CONTEXT_SCHEMA_VERSION changed — update docs/SCHEMA.md");
    assert_eq!(CONTEXT_BUNDLE_SCHEMA_VERSION, 2, "CONTEXT_BUNDLE_SCHEMA_VERSION changed — update docs/SCHEMA.md");
}

// =============================================================================
// LangReceipt schema validation
// =============================================================================

#[test]
fn lang_receipt_json_contains_required_envelope_fields() {
    let receipt = sample_lang_receipt();
    let json: Value = serde_json::to_value(&receipt).unwrap();

    assert_eq!(json["schema_version"], SCHEMA_VERSION);
    assert!(json["generated_at_ms"].is_number());
    assert_eq!(json["tool"]["name"], "tokmd");
    assert!(json["tool"]["version"].is_string());
    assert_eq!(json["mode"], "lang");
    assert!(json["status"].is_string());
    assert!(json["warnings"].is_array());
    assert!(json["scan"].is_object());
    assert!(json["args"].is_object());
}

#[test]
fn lang_receipt_json_contains_report_fields() {
    let receipt = sample_lang_receipt();
    let json: Value = serde_json::to_value(&receipt).unwrap();

    // LangReport is flattened, so rows/total appear at top level
    assert!(json["rows"].is_array());
    assert!(json["total"].is_object());
    assert!(json["with_files"].is_boolean());
    assert!(json["children"].is_string());
    assert!(json["top"].is_number());
}

#[test]
fn lang_receipt_scan_args_fields_present() {
    let receipt = sample_lang_receipt();
    let json: Value = serde_json::to_value(&receipt).unwrap();
    let scan = &json["scan"];

    assert!(scan["paths"].is_array());
    assert!(scan["excluded"].is_array());
    assert!(scan["config"].is_string());
    assert!(scan["hidden"].is_boolean());
    assert!(scan["no_ignore"].is_boolean());
    assert!(scan["no_ignore_parent"].is_boolean());
    assert!(scan["no_ignore_dot"].is_boolean());
    assert!(scan["no_ignore_vcs"].is_boolean());
    assert!(scan["treat_doc_strings_as_comments"].is_boolean());
}

#[test]
fn lang_receipt_roundtrip() {
    let receipt = sample_lang_receipt();
    let json_str = serde_json::to_string(&receipt).unwrap();
    let deserialized: LangReceipt = serde_json::from_str(&json_str).unwrap();

    assert_eq!(deserialized.schema_version, receipt.schema_version);
    assert_eq!(deserialized.generated_at_ms, receipt.generated_at_ms);
    assert_eq!(deserialized.mode, receipt.mode);
    assert_eq!(deserialized.tool.name, receipt.tool.name);
    assert_eq!(deserialized.tool.version, receipt.tool.version);
    assert_eq!(deserialized.report.rows.len(), receipt.report.rows.len());
    assert_eq!(deserialized.report.total, receipt.report.total);
}

#[test]
fn lang_receipt_row_fields_complete() {
    let receipt = sample_lang_receipt();
    let json: Value = serde_json::to_value(&receipt).unwrap();
    let row = &json["rows"][0];

    assert_eq!(row["lang"], "Rust");
    assert!(row["code"].is_number());
    assert!(row["lines"].is_number());
    assert!(row["files"].is_number());
    assert!(row["bytes"].is_number());
    assert!(row["tokens"].is_number());
    assert!(row["avg_lines"].is_number());
}

// =============================================================================
// ModuleReceipt schema validation
// =============================================================================

#[test]
fn module_receipt_json_contains_required_envelope_fields() {
    let receipt = sample_module_receipt();
    let json: Value = serde_json::to_value(&receipt).unwrap();

    assert_eq!(json["schema_version"], SCHEMA_VERSION);
    assert!(json["generated_at_ms"].is_number());
    assert_eq!(json["tool"]["name"], "tokmd");
    assert_eq!(json["mode"], "module");
    assert!(json["status"].is_string());
    assert!(json["warnings"].is_array());
    assert!(json["scan"].is_object());
    assert!(json["args"].is_object());
}

#[test]
fn module_receipt_json_contains_report_fields() {
    let receipt = sample_module_receipt();
    let json: Value = serde_json::to_value(&receipt).unwrap();

    // ModuleReport is flattened
    assert!(json["rows"].is_array());
    assert!(json["total"].is_object());
    assert!(json["module_roots"].is_array());
    assert!(json["module_depth"].is_number());
    assert!(json["children"].is_string());
    assert!(json["top"].is_number());
}

#[test]
fn module_receipt_roundtrip() {
    let receipt = sample_module_receipt();
    let json_str = serde_json::to_string(&receipt).unwrap();
    let deserialized: ModuleReceipt = serde_json::from_str(&json_str).unwrap();

    assert_eq!(deserialized.schema_version, receipt.schema_version);
    assert_eq!(deserialized.generated_at_ms, receipt.generated_at_ms);
    assert_eq!(deserialized.mode, receipt.mode);
    assert_eq!(deserialized.report.rows.len(), receipt.report.rows.len());
    assert_eq!(deserialized.report.total, receipt.report.total);
}

#[test]
fn module_receipt_row_fields_complete() {
    let receipt = sample_module_receipt();
    let json: Value = serde_json::to_value(&receipt).unwrap();
    let row = &json["rows"][0];

    assert_eq!(row["module"], "src");
    assert!(row["code"].is_number());
    assert!(row["lines"].is_number());
    assert!(row["files"].is_number());
    assert!(row["bytes"].is_number());
    assert!(row["tokens"].is_number());
    assert!(row["avg_lines"].is_number());
}

// =============================================================================
// ExportReceipt schema validation
// =============================================================================

#[test]
fn export_receipt_json_contains_required_envelope_fields() {
    let receipt = sample_export_receipt();
    let json: Value = serde_json::to_value(&receipt).unwrap();

    assert_eq!(json["schema_version"], SCHEMA_VERSION);
    assert!(json["generated_at_ms"].is_number());
    assert_eq!(json["tool"]["name"], "tokmd");
    assert_eq!(json["mode"], "export");
    assert!(json["status"].is_string());
    assert!(json["warnings"].is_array());
    assert!(json["scan"].is_object());
    assert!(json["args"].is_object());
}

#[test]
fn export_receipt_json_contains_data_fields() {
    let receipt = sample_export_receipt();
    let json: Value = serde_json::to_value(&receipt).unwrap();

    // ExportData is flattened
    assert!(json["rows"].is_array());
    assert!(json["module_roots"].is_array());
    assert!(json["module_depth"].is_number());
    assert!(json["children"].is_string());
}

#[test]
fn export_receipt_args_fields_present() {
    let receipt = sample_export_receipt();
    let json: Value = serde_json::to_value(&receipt).unwrap();
    let args = &json["args"];

    assert!(args["format"].is_string());
    assert!(args["module_roots"].is_array());
    assert!(args["module_depth"].is_number());
    assert!(args["children"].is_string());
    assert!(args["min_code"].is_number());
    assert!(args["max_rows"].is_number());
    assert!(args["redact"].is_string());
}

#[test]
fn export_receipt_roundtrip() {
    let receipt = sample_export_receipt();
    let json_str = serde_json::to_string(&receipt).unwrap();
    let deserialized: ExportReceipt = serde_json::from_str(&json_str).unwrap();

    assert_eq!(deserialized.schema_version, receipt.schema_version);
    assert_eq!(deserialized.generated_at_ms, receipt.generated_at_ms);
    assert_eq!(deserialized.mode, receipt.mode);
    assert_eq!(deserialized.data.rows.len(), receipt.data.rows.len());
}

// =============================================================================
// DiffReceipt schema validation
// =============================================================================

#[test]
fn diff_receipt_json_contains_required_fields() {
    let receipt = sample_diff_receipt();
    let json: Value = serde_json::to_value(&receipt).unwrap();

    assert_eq!(json["schema_version"], SCHEMA_VERSION);
    assert!(json["generated_at_ms"].is_number());
    assert_eq!(json["tool"]["name"], "tokmd");
    assert_eq!(json["mode"], "diff");
    assert!(json["from_source"].is_string());
    assert!(json["to_source"].is_string());
    assert!(json["diff_rows"].is_array());
    assert!(json["totals"].is_object());
}

#[test]
fn diff_receipt_roundtrip() {
    let receipt = sample_diff_receipt();
    let json_str = serde_json::to_string(&receipt).unwrap();
    let deserialized: DiffReceipt = serde_json::from_str(&json_str).unwrap();

    assert_eq!(deserialized.schema_version, receipt.schema_version);
    assert_eq!(deserialized.mode, receipt.mode);
    assert_eq!(deserialized.from_source, receipt.from_source);
    assert_eq!(deserialized.to_source, receipt.to_source);
    assert_eq!(deserialized.totals, receipt.totals);
}

#[test]
fn diff_totals_default_is_all_zeros() {
    let totals = DiffTotals::default();
    let json: Value = serde_json::to_value(&totals).unwrap();

    for key in [
        "old_code",
        "new_code",
        "delta_code",
        "old_lines",
        "new_lines",
        "delta_lines",
        "old_files",
        "new_files",
        "delta_files",
        "old_bytes",
        "new_bytes",
        "delta_bytes",
        "old_tokens",
        "new_tokens",
        "delta_tokens",
    ] {
        assert_eq!(json[key], 0, "DiffTotals.{key} should default to 0");
    }
}

// =============================================================================
// ToolInfo envelope metadata
// =============================================================================

#[test]
fn tool_info_current_produces_valid_metadata() {
    let info = ToolInfo::current();

    assert_eq!(info.name, "tokmd");
    assert!(!info.version.is_empty(), "version should not be empty");

    let json: Value = serde_json::to_value(&info).unwrap();
    assert_eq!(json["name"], "tokmd");
    assert!(json["version"].is_string());
}

#[test]
fn tool_info_roundtrip() {
    let info = ToolInfo::current();
    let json_str = serde_json::to_string(&info).unwrap();
    let deserialized: ToolInfo = serde_json::from_str(&json_str).unwrap();

    assert_eq!(deserialized.name, info.name);
    assert_eq!(deserialized.version, info.version);
}

// =============================================================================
// Totals structure validation
// =============================================================================

#[test]
fn totals_json_has_all_fields() {
    let totals = sample_totals();
    let json: Value = serde_json::to_value(&totals).unwrap();

    assert!(json["code"].is_number());
    assert!(json["lines"].is_number());
    assert!(json["files"].is_number());
    assert!(json["bytes"].is_number());
    assert!(json["tokens"].is_number());
    assert!(json["avg_lines"].is_number());

    // No extra fields
    let obj = json.as_object().unwrap();
    assert_eq!(obj.len(), 6, "Totals should have exactly 6 fields");
}

#[test]
fn totals_roundtrip_preserves_values() {
    let totals = sample_totals();
    let json_str = serde_json::to_string(&totals).unwrap();
    let deserialized: Totals = serde_json::from_str(&json_str).unwrap();

    assert_eq!(deserialized, totals);
}

// =============================================================================
// ScanStatus serialization
// =============================================================================

#[test]
fn scan_status_serializes_to_snake_case() {
    let complete_json = serde_json::to_string(&ScanStatus::Complete).unwrap();
    let partial_json = serde_json::to_string(&ScanStatus::Partial).unwrap();

    assert_eq!(complete_json, "\"complete\"");
    assert_eq!(partial_json, "\"partial\"");
}

#[test]
fn scan_status_roundtrip() {
    for status in [ScanStatus::Complete, ScanStatus::Partial] {
        let json_str = serde_json::to_string(&status).unwrap();
        let deserialized: ScanStatus = serde_json::from_str(&json_str).unwrap();
        let re_serialized = serde_json::to_string(&deserialized).unwrap();
        assert_eq!(json_str, re_serialized);
    }
}

// =============================================================================
// Envelope consistency across receipt types
// =============================================================================

#[test]
fn all_receipts_share_envelope_structure() {
    let lang_json: Value = serde_json::to_value(&sample_lang_receipt()).unwrap();
    let module_json: Value = serde_json::to_value(&sample_module_receipt()).unwrap();
    let export_json: Value = serde_json::to_value(&sample_export_receipt()).unwrap();
    let diff_json: Value = serde_json::to_value(&sample_diff_receipt()).unwrap();

    let envelope_fields = ["schema_version", "generated_at_ms", "tool", "mode"];

    for field in &envelope_fields {
        assert!(
            !lang_json[field].is_null(),
            "LangReceipt missing envelope field: {field}"
        );
        assert!(
            !module_json[field].is_null(),
            "ModuleReceipt missing envelope field: {field}"
        );
        assert!(
            !export_json[field].is_null(),
            "ExportReceipt missing envelope field: {field}"
        );
        assert!(
            !diff_json[field].is_null(),
            "DiffReceipt missing envelope field: {field}"
        );
    }

    // All receipts use the same schema_version
    assert_eq!(lang_json["schema_version"], SCHEMA_VERSION);
    assert_eq!(module_json["schema_version"], SCHEMA_VERSION);
    assert_eq!(export_json["schema_version"], SCHEMA_VERSION);
    assert_eq!(diff_json["schema_version"], SCHEMA_VERSION);
}

// =============================================================================
// JSON stability: no unexpected null fields
// =============================================================================

#[test]
fn lang_receipt_json_has_no_null_required_fields() {
    let json: Value = serde_json::to_value(&sample_lang_receipt()).unwrap();
    let obj = json.as_object().unwrap();

    for (key, value) in obj {
        assert!(
            !value.is_null(),
            "LangReceipt field '{key}' should not be null"
        );
    }
}

#[test]
fn module_receipt_json_has_no_null_required_fields() {
    let json: Value = serde_json::to_value(&sample_module_receipt()).unwrap();
    let obj = json.as_object().unwrap();

    for (key, value) in obj {
        assert!(
            !value.is_null(),
            "ModuleReceipt field '{key}' should not be null"
        );
    }
}

#[test]
fn export_receipt_json_has_no_null_required_fields() {
    let json: Value = serde_json::to_value(&sample_export_receipt()).unwrap();
    let obj = json.as_object().unwrap();

    for (key, value) in obj {
        assert!(
            !value.is_null(),
            "ExportReceipt field '{key}' should not be null"
        );
    }
}
