//! Schema compliance tests for core receipt types.
//!
//! These tests verify that receipt structures conform to their documented schemas
//! and that schema version constants are correctly maintained.

use serde_json::Value;
use tokmd_types::{
    CONTEXT_BUNDLE_SCHEMA_VERSION, CONTEXT_SCHEMA_VERSION, ChildIncludeMode, ChildrenMode,
    ConfigMode, ContextReceipt, DiffReceipt, DiffTotals, ExportArgsMeta, ExportData, ExportFormat,
    ExportReceipt, FileKind, FileRow, HANDOFF_SCHEMA_VERSION, LangArgsMeta, LangReceipt,
    LangReport, LangRow, ModuleArgsMeta, ModuleReceipt, ModuleReport, ModuleRow, RedactMode,
    SCHEMA_VERSION, ScanArgs, ScanStatus, ToolInfo, Totals, cockpit::COCKPIT_SCHEMA_VERSION,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn sample_totals() -> Totals {
    Totals {
        code: 100,
        lines: 150,
        files: 5,
        bytes: 5000,
        tokens: 1000,
        avg_lines: 30,
    }
}

fn sample_tool_info() -> ToolInfo {
    ToolInfo {
        name: "tokmd".into(),
        version: "0.0.0-test".into(),
    }
}

fn sample_scan_args() -> ScanArgs {
    ScanArgs {
        paths: vec![".".into()],
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

fn sample_lang_receipt() -> LangReceipt {
    LangReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 1_700_000_000_000,
        tool: sample_tool_info(),
        mode: "lang".into(),
        status: ScanStatus::Complete,
        warnings: vec![],
        scan: sample_scan_args(),
        args: LangArgsMeta {
            format: "json".into(),
            top: 0,
            with_files: true,
            children: ChildrenMode::Collapse,
        },
        report: LangReport {
            rows: vec![LangRow {
                lang: "Rust".into(),
                code: 100,
                lines: 150,
                files: 5,
                bytes: 5000,
                tokens: 1000,
                avg_lines: 30,
            }],
            total: sample_totals(),
            with_files: true,
            children: ChildrenMode::Collapse,
            top: 0,
        },
    }
}

fn sample_module_receipt() -> ModuleReceipt {
    ModuleReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 1_700_000_000_000,
        tool: sample_tool_info(),
        mode: "module".into(),
        status: ScanStatus::Complete,
        warnings: vec![],
        scan: sample_scan_args(),
        args: ModuleArgsMeta {
            format: "json".into(),
            module_roots: vec![],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
            top: 0,
        },
        report: ModuleReport {
            rows: vec![ModuleRow {
                module: "src".into(),
                code: 100,
                lines: 150,
                files: 5,
                bytes: 5000,
                tokens: 1000,
                avg_lines: 30,
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
        generated_at_ms: 1_700_000_000_000,
        tool: sample_tool_info(),
        mode: "export".into(),
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
            rows: vec![FileRow {
                path: "src/main.rs".into(),
                module: "src".into(),
                lang: "Rust".into(),
                kind: FileKind::Parent,
                code: 100,
                comments: 20,
                blanks: 30,
                lines: 150,
                bytes: 5000,
                tokens: 1000,
            }],
            module_roots: vec![],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
        },
    }
}

fn sample_diff_receipt() -> DiffReceipt {
    DiffReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 1_700_000_000_000,
        tool: sample_tool_info(),
        mode: "diff".into(),
        from_source: "a.json".into(),
        to_source: "b.json".into(),
        diff_rows: vec![],
        totals: DiffTotals::default(),
    }
}

fn sample_context_receipt() -> ContextReceipt {
    ContextReceipt {
        schema_version: CONTEXT_SCHEMA_VERSION,
        generated_at_ms: 1_700_000_000_000,
        tool: sample_tool_info(),
        mode: "context".into(),
        budget_tokens: 100_000,
        used_tokens: 50_000,
        utilization_pct: 50.0,
        strategy: "greedy".into(),
        rank_by: "tokens".into(),
        file_count: 0,
        files: vec![],
        rank_by_effective: None,
        fallback_reason: None,
        excluded_by_policy: vec![],
        token_estimation: None,
        bundle_audit: None,
    }
}

// ---------------------------------------------------------------------------
// 1. Schema version constants are positive integers
// ---------------------------------------------------------------------------

#[test]
fn schema_version_is_positive() {
    assert!(SCHEMA_VERSION > 0);
}

#[test]
fn cockpit_schema_version_is_positive() {
    assert!(COCKPIT_SCHEMA_VERSION > 0);
}

#[test]
fn handoff_schema_version_is_positive() {
    assert!(HANDOFF_SCHEMA_VERSION > 0);
}

#[test]
fn context_schema_version_is_positive() {
    assert!(CONTEXT_SCHEMA_VERSION > 0);
}

#[test]
fn context_bundle_schema_version_is_positive() {
    assert!(CONTEXT_BUNDLE_SCHEMA_VERSION > 0);
}

// ---------------------------------------------------------------------------
// 2. Schema version constants match documented values
// ---------------------------------------------------------------------------

#[test]
fn schema_version_matches_documented_value() {
    assert_eq!(SCHEMA_VERSION, 2);
    assert_eq!(COCKPIT_SCHEMA_VERSION, 3);
    assert_eq!(HANDOFF_SCHEMA_VERSION, 5);
    assert_eq!(CONTEXT_SCHEMA_VERSION, 4);
    assert_eq!(CONTEXT_BUNDLE_SCHEMA_VERSION, 2);
}

// ---------------------------------------------------------------------------
// 3. LangReceipt JSON has required fields
// ---------------------------------------------------------------------------

#[test]
fn lang_receipt_json_has_required_fields() {
    let receipt = sample_lang_receipt();
    let json: Value = serde_json::to_value(&receipt).unwrap();
    let obj = json.as_object().unwrap();

    assert!(obj.contains_key("schema_version"));
    assert!(obj.contains_key("tool"));
    // LangReport is flattened, so `total` and `rows` appear at top level
    assert!(obj.contains_key("total"));
    assert!(obj.contains_key("rows"));
    assert!(obj.contains_key("mode"));
    assert!(obj.contains_key("status"));
    assert!(obj.contains_key("scan"));
    assert!(obj.contains_key("args"));
}

#[test]
fn lang_receipt_schema_version_in_json() {
    let receipt = sample_lang_receipt();
    let json: Value = serde_json::to_value(&receipt).unwrap();
    assert_eq!(json["schema_version"], SCHEMA_VERSION);
}

// ---------------------------------------------------------------------------
// 4. ModuleReceipt JSON has required fields
// ---------------------------------------------------------------------------

#[test]
fn module_receipt_json_has_required_fields() {
    let receipt = sample_module_receipt();
    let json: Value = serde_json::to_value(&receipt).unwrap();
    let obj = json.as_object().unwrap();

    assert!(obj.contains_key("schema_version"));
    assert!(obj.contains_key("tool"));
    // ModuleReport is flattened
    assert!(obj.contains_key("total"));
    assert!(obj.contains_key("rows"));
    assert!(obj.contains_key("mode"));
}

#[test]
fn module_receipt_schema_version_in_json() {
    let receipt = sample_module_receipt();
    let json: Value = serde_json::to_value(&receipt).unwrap();
    assert_eq!(json["schema_version"], SCHEMA_VERSION);
}

// ---------------------------------------------------------------------------
// 5. ExportReceipt JSON has required fields
// ---------------------------------------------------------------------------

#[test]
fn export_receipt_json_has_required_fields() {
    let receipt = sample_export_receipt();
    let json: Value = serde_json::to_value(&receipt).unwrap();
    let obj = json.as_object().unwrap();

    assert!(obj.contains_key("schema_version"));
    assert!(obj.contains_key("tool"));
    // ExportData is flattened
    assert!(obj.contains_key("rows"));
    assert!(obj.contains_key("mode"));
}

// ---------------------------------------------------------------------------
// 6. DiffReceipt JSON has required fields
// ---------------------------------------------------------------------------

#[test]
fn diff_receipt_json_has_required_fields() {
    let receipt = sample_diff_receipt();
    let json: Value = serde_json::to_value(&receipt).unwrap();
    let obj = json.as_object().unwrap();

    assert!(obj.contains_key("schema_version"));
    assert!(obj.contains_key("totals"));
    assert!(obj.contains_key("diff_rows"));
    assert!(obj.contains_key("from_source"));
    assert!(obj.contains_key("to_source"));
}

// ---------------------------------------------------------------------------
// 7. ContextReceipt JSON has required fields
// ---------------------------------------------------------------------------

#[test]
fn context_receipt_json_has_required_fields() {
    let receipt = sample_context_receipt();
    let json: Value = serde_json::to_value(&receipt).unwrap();
    let obj = json.as_object().unwrap();

    assert!(obj.contains_key("schema_version"));
    assert_eq!(json["schema_version"], CONTEXT_SCHEMA_VERSION);
    assert!(obj.contains_key("budget_tokens"));
    assert!(obj.contains_key("used_tokens"));
    assert!(obj.contains_key("files"));
}

// ---------------------------------------------------------------------------
// 8. Field type correctness
// ---------------------------------------------------------------------------

#[test]
fn lang_receipt_rows_is_array_total_is_object() {
    let receipt = sample_lang_receipt();
    let json: Value = serde_json::to_value(&receipt).unwrap();

    assert!(json["rows"].is_array());
    assert!(json["total"].is_object());
    assert!(json["schema_version"].is_number());
    assert!(json["tool"].is_object());
    assert!(json["warnings"].is_array());
}

#[test]
fn diff_receipt_diff_rows_is_array_totals_is_object() {
    let receipt = sample_diff_receipt();
    let json: Value = serde_json::to_value(&receipt).unwrap();

    assert!(json["diff_rows"].is_array());
    assert!(json["totals"].is_object());
}

// ---------------------------------------------------------------------------
// 9. Serde roundtrip correctness
// ---------------------------------------------------------------------------

#[test]
fn lang_receipt_roundtrip() {
    let receipt = sample_lang_receipt();
    let json = serde_json::to_string(&receipt).unwrap();
    let back: LangReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema_version, SCHEMA_VERSION);
    assert_eq!(back.mode, "lang");
    assert_eq!(back.report.rows.len(), 1);
    assert_eq!(back.report.rows[0].lang, "Rust");
}

#[test]
fn module_receipt_roundtrip() {
    let receipt = sample_module_receipt();
    let json = serde_json::to_string(&receipt).unwrap();
    let back: ModuleReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema_version, SCHEMA_VERSION);
    assert_eq!(back.mode, "module");
    assert_eq!(back.report.rows.len(), 1);
}

#[test]
fn export_receipt_roundtrip() {
    let receipt = sample_export_receipt();
    let json = serde_json::to_string(&receipt).unwrap();
    let back: ExportReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema_version, SCHEMA_VERSION);
    assert_eq!(back.data.rows.len(), 1);
    assert_eq!(back.data.rows[0].path, "src/main.rs");
}

#[test]
fn diff_receipt_roundtrip() {
    let receipt = sample_diff_receipt();
    let json = serde_json::to_string(&receipt).unwrap();
    let back: DiffReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema_version, SCHEMA_VERSION);
    assert_eq!(back.from_source, "a.json");
    assert_eq!(back.to_source, "b.json");
}

#[test]
fn context_receipt_roundtrip() {
    let receipt = sample_context_receipt();
    let json = serde_json::to_string(&receipt).unwrap();
    let back: ContextReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema_version, CONTEXT_SCHEMA_VERSION);
    assert_eq!(back.budget_tokens, 100_000);
}

// ---------------------------------------------------------------------------
// 10. ToolInfo structure
// ---------------------------------------------------------------------------

#[test]
fn tool_info_json_has_name_and_version() {
    let ti = sample_tool_info();
    let json: Value = serde_json::to_value(&ti).unwrap();
    let obj = json.as_object().unwrap();

    assert!(obj.contains_key("name"));
    assert!(obj.contains_key("version"));
    assert_eq!(json["name"], "tokmd");
}

#[test]
fn tool_info_current_returns_non_empty() {
    let ti = ToolInfo::current();
    assert_eq!(ti.name, "tokmd");
    assert!(!ti.version.is_empty());
}
