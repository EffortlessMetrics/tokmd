//! Schema versioning and backward compatibility contract tests for tokmd-types.

use serde_json::Value;
use tokmd_types::{
    CONTEXT_BUNDLE_SCHEMA_VERSION, CONTEXT_SCHEMA_VERSION, ChildIncludeMode, ChildrenMode,
    ConfigMode, DiffReceipt, DiffTotals, ExportArgsMeta, ExportData, ExportFormat, ExportReceipt,
    FileKind, FileRow, HANDOFF_SCHEMA_VERSION, LangArgsMeta, LangReceipt, LangReport, LangRow,
    ModuleArgsMeta, ModuleReceipt, ModuleReport, ModuleRow, RedactMode, SCHEMA_VERSION, ScanArgs,
    ScanStatus, ToolInfo, Totals,
};

// ── Helpers ──────────────────────────────────────────────────────────────

fn sample_tool_info() -> ToolInfo {
    ToolInfo {
        name: "tokmd".into(),
        version: "1.0.0".into(),
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

fn sample_totals() -> Totals {
    Totals {
        code: 100,
        lines: 150,
        files: 5,
        bytes: 4000,
        tokens: 1000,
        avg_lines: 30,
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
            with_files: false,
            children: ChildrenMode::Collapse,
        },
        report: LangReport {
            rows: vec![LangRow {
                lang: "Rust".into(),
                code: 100,
                lines: 150,
                files: 5,
                bytes: 4000,
                tokens: 1000,
                avg_lines: 30,
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
        generated_at_ms: 1_700_000_000_000,
        tool: sample_tool_info(),
        mode: "module".into(),
        status: ScanStatus::Complete,
        warnings: vec![],
        scan: sample_scan_args(),
        args: ModuleArgsMeta {
            format: "json".into(),
            module_roots: vec!["crates".into()],
            module_depth: 2,
            children: ChildIncludeMode::Separate,
            top: 0,
        },
        report: ModuleReport {
            rows: vec![ModuleRow {
                module: "crates/tokmd-types".into(),
                code: 100,
                lines: 150,
                files: 5,
                bytes: 4000,
                tokens: 1000,
                avg_lines: 30,
            }],
            total: sample_totals(),
            module_roots: vec!["crates".into()],
            module_depth: 2,
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
            format: ExportFormat::Jsonl,
            module_roots: vec!["crates".into()],
            module_depth: 2,
            children: ChildIncludeMode::Separate,
            min_code: 0,
            max_rows: 0,
            redact: RedactMode::None,
            strip_prefix: None,
            strip_prefix_redacted: false,
        },
        data: ExportData {
            rows: vec![FileRow {
                path: "src/lib.rs".into(),
                module: "crates/tokmd-types".into(),
                lang: "Rust".into(),
                kind: FileKind::Parent,
                code: 100,
                comments: 20,
                blanks: 30,
                lines: 150,
                bytes: 4000,
                tokens: 1000,
            }],
            module_roots: vec!["crates".into()],
            module_depth: 2,
            children: ChildIncludeMode::Separate,
        },
    }
}

// ── Schema version constants ─────────────────────────────────────────────

#[test]
fn schema_version_is_positive() {
    let v = SCHEMA_VERSION;
    assert!(v > 0, "SCHEMA_VERSION must be a positive integer");
}

#[test]
fn schema_version_pinned() {
    assert_eq!(SCHEMA_VERSION, 2);
}

#[test]
fn all_schema_versions_are_positive() {
    let h = HANDOFF_SCHEMA_VERSION;
    assert!(h > 0);
    let cb = CONTEXT_BUNDLE_SCHEMA_VERSION;
    assert!(cb > 0);
    let cs = CONTEXT_SCHEMA_VERSION;
    assert!(cs > 0);
}

// ── Default values ───────────────────────────────────────────────────────

#[test]
fn config_mode_default_is_auto() {
    assert_eq!(ConfigMode::default(), ConfigMode::Auto);
}

#[test]
fn tool_info_default_has_empty_fields() {
    let ti = ToolInfo::default();
    assert!(ti.name.is_empty());
    assert!(ti.version.is_empty());
}

#[test]
fn diff_totals_default_all_zero() {
    let dt = DiffTotals::default();
    assert_eq!(dt.old_code, 0);
    assert_eq!(dt.new_code, 0);
    assert_eq!(dt.delta_code, 0);
    assert_eq!(dt.old_files, 0);
    assert_eq!(dt.new_files, 0);
    assert_eq!(dt.delta_files, 0);
    assert_eq!(dt.old_tokens, 0);
    assert_eq!(dt.new_tokens, 0);
    assert_eq!(dt.delta_tokens, 0);
}

// ── JSON serialization round-trips ───────────────────────────────────────

#[test]
fn lang_receipt_json_roundtrip() {
    let receipt = sample_lang_receipt();
    let json = serde_json::to_string(&receipt).unwrap();
    let back: LangReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema_version, SCHEMA_VERSION);
    assert_eq!(back.mode, "lang");
    assert_eq!(back.report.rows.len(), 1);
    assert_eq!(back.report.rows[0].lang, "Rust");
}

#[test]
fn module_receipt_json_roundtrip() {
    let receipt = sample_module_receipt();
    let json = serde_json::to_string(&receipt).unwrap();
    let back: ModuleReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema_version, SCHEMA_VERSION);
    assert_eq!(back.mode, "module");
    assert_eq!(back.report.rows.len(), 1);
}

#[test]
fn export_receipt_json_roundtrip() {
    let receipt = sample_export_receipt();
    let json = serde_json::to_string(&receipt).unwrap();
    let back: ExportReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema_version, SCHEMA_VERSION);
    assert_eq!(back.mode, "export");
    assert_eq!(back.data.rows.len(), 1);
    assert_eq!(back.data.rows[0].path, "src/lib.rs");
}

#[test]
fn diff_receipt_json_roundtrip() {
    let receipt = DiffReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 1_700_000_000_000,
        tool: sample_tool_info(),
        mode: "diff".into(),
        from_source: "v1.0".into(),
        to_source: "v2.0".into(),
        diff_rows: vec![],
        totals: DiffTotals::default(),
    };
    let json = serde_json::to_string(&receipt).unwrap();
    let back: DiffReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema_version, SCHEMA_VERSION);
    assert_eq!(back.from_source, "v1.0");
    assert_eq!(back.to_source, "v2.0");
}

// ── schema_version field appears in serialized output ────────────────────

#[test]
fn schema_version_field_in_lang_json() {
    let receipt = sample_lang_receipt();
    let json = serde_json::to_string(&receipt).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["schema_version"], SCHEMA_VERSION);
}

#[test]
fn schema_version_field_in_module_json() {
    let receipt = sample_module_receipt();
    let json = serde_json::to_string(&receipt).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["schema_version"], SCHEMA_VERSION);
}

#[test]
fn schema_version_field_in_export_json() {
    let receipt = sample_export_receipt();
    let json = serde_json::to_string(&receipt).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["schema_version"], SCHEMA_VERSION);
}

// ── Enum serde stability ─────────────────────────────────────────────────

#[test]
fn children_mode_serde_values() {
    assert_eq!(
        serde_json::to_string(&ChildrenMode::Collapse).unwrap(),
        "\"collapse\""
    );
    assert_eq!(
        serde_json::to_string(&ChildrenMode::Separate).unwrap(),
        "\"separate\""
    );
}

#[test]
fn export_format_serde_values() {
    assert_eq!(
        serde_json::to_string(&ExportFormat::Jsonl).unwrap(),
        "\"jsonl\""
    );
    assert_eq!(
        serde_json::to_string(&ExportFormat::Csv).unwrap(),
        "\"csv\""
    );
    assert_eq!(
        serde_json::to_string(&ExportFormat::Json).unwrap(),
        "\"json\""
    );
    assert_eq!(
        serde_json::to_string(&ExportFormat::Cyclonedx).unwrap(),
        "\"cyclonedx\""
    );
}

#[test]
fn redact_mode_serde_values() {
    assert_eq!(
        serde_json::to_string(&RedactMode::None).unwrap(),
        "\"none\""
    );
    assert_eq!(
        serde_json::to_string(&RedactMode::Paths).unwrap(),
        "\"paths\""
    );
    assert_eq!(serde_json::to_string(&RedactMode::All).unwrap(), "\"all\"");
}

// ── Property tests ───────────────────────────────────────────────────────

mod properties {
    use proptest::prelude::*;
    use tokmd_types::{
        ChildrenMode, LangArgsMeta, LangReceipt, LangReport, LangRow, SCHEMA_VERSION, ScanArgs,
        ScanStatus, ToolInfo, Totals,
    };

    fn arb_lang_receipt() -> impl Strategy<Value = LangReceipt> {
        (
            any::<u128>(),
            proptest::collection::vec("[a-zA-Z]{2,10}", 0..5),
        )
            .prop_map(|(ts, langs)| {
                let rows: Vec<LangRow> = langs
                    .iter()
                    .map(|l| LangRow {
                        lang: l.clone(),
                        code: 10,
                        lines: 15,
                        files: 1,
                        bytes: 400,
                        tokens: 100,
                        avg_lines: 15,
                    })
                    .collect();
                let total = Totals {
                    code: rows.iter().map(|r| r.code).sum(),
                    lines: rows.iter().map(|r| r.lines).sum(),
                    files: rows.iter().map(|r| r.files).sum(),
                    bytes: rows.iter().map(|r| r.bytes).sum(),
                    tokens: rows.iter().map(|r| r.tokens).sum(),
                    avg_lines: 15,
                };
                LangReceipt {
                    schema_version: SCHEMA_VERSION,
                    generated_at_ms: ts,
                    tool: ToolInfo {
                        name: "tokmd".into(),
                        version: "1.0.0".into(),
                    },
                    mode: "lang".into(),
                    status: ScanStatus::Complete,
                    warnings: vec![],
                    scan: ScanArgs {
                        paths: vec![".".into()],
                        excluded: vec![],
                        excluded_redacted: false,
                        config: tokmd_types::ConfigMode::Auto,
                        hidden: false,
                        no_ignore: false,
                        no_ignore_parent: false,
                        no_ignore_dot: false,
                        no_ignore_vcs: false,
                        treat_doc_strings_as_comments: false,
                    },
                    args: LangArgsMeta {
                        format: "json".into(),
                        top: 0,
                        with_files: false,
                        children: ChildrenMode::Collapse,
                    },
                    report: LangReport {
                        rows,
                        total,
                        with_files: false,
                        children: ChildrenMode::Collapse,
                        top: 0,
                    },
                }
            })
    }

    proptest! {
        #[test]
        fn receipt_json_roundtrip(receipt in arb_lang_receipt()) {
            let json = serde_json::to_string(&receipt).unwrap();
            let back: LangReceipt = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(back.schema_version, SCHEMA_VERSION);
            prop_assert_eq!(back.report.rows.len(), receipt.report.rows.len());
        }
    }
}
