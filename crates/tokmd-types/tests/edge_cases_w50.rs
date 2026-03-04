//! Edge-case and boundary-condition tests for tokmd-types.

use tokmd_types::cockpit::COCKPIT_SCHEMA_VERSION;
use tokmd_types::{
    CONTEXT_BUNDLE_SCHEMA_VERSION, CONTEXT_SCHEMA_VERSION, ChildIncludeMode, ChildrenMode,
    ConfigMode, ExportData, ExportFormat, FileKind, FileRow, HANDOFF_SCHEMA_VERSION, LangReport,
    LangRow, ModuleReport, ModuleRow, RedactMode, SCHEMA_VERSION, TableFormat, Totals,
};

// ---------------------------------------------------------------------------
// Schema version constants
// ---------------------------------------------------------------------------

#[test]
fn schema_version_is_nonzero() {
    assert!(SCHEMA_VERSION > 0);
}

#[test]
fn cockpit_schema_version_is_nonzero() {
    assert!(COCKPIT_SCHEMA_VERSION > 0);
}

#[test]
fn handoff_schema_version_is_nonzero() {
    assert!(HANDOFF_SCHEMA_VERSION > 0);
}

#[test]
fn context_schema_version_is_nonzero() {
    assert!(CONTEXT_SCHEMA_VERSION > 0);
}

#[test]
fn context_bundle_schema_version_is_nonzero() {
    assert!(CONTEXT_BUNDLE_SCHEMA_VERSION > 0);
}

// ---------------------------------------------------------------------------
// LangRow with extreme values
// ---------------------------------------------------------------------------

#[test]
fn lang_row_u64_max_code_no_overflow() {
    let row = LangRow {
        lang: "Rust".to_string(),
        code: usize::MAX,
        lines: usize::MAX,
        files: 1,
        bytes: 0,
        tokens: 0,
        avg_lines: usize::MAX,
    };
    let json = serde_json::to_string(&row).unwrap();
    let back: LangRow = serde_json::from_str(&json).unwrap();
    assert_eq!(back.code, usize::MAX);
    assert_eq!(back.lines, usize::MAX);
}

#[test]
fn lang_row_all_zeros() {
    let row = LangRow {
        lang: String::new(),
        code: 0,
        lines: 0,
        files: 0,
        bytes: 0,
        tokens: 0,
        avg_lines: 0,
    };
    let json = serde_json::to_string(&row).unwrap();
    let back: LangRow = serde_json::from_str(&json).unwrap();
    assert_eq!(back, row);
}

// ---------------------------------------------------------------------------
// Receipt with empty rows
// ---------------------------------------------------------------------------

#[test]
fn lang_report_empty_rows() {
    let report = LangReport {
        rows: vec![],
        total: Totals {
            code: 0,
            lines: 0,
            files: 0,
            bytes: 0,
            tokens: 0,
            avg_lines: 0,
        },
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let json = serde_json::to_string(&report).unwrap();
    let back: LangReport = serde_json::from_str(&json).unwrap();
    assert!(back.rows.is_empty());
    assert_eq!(back.total.code, 0);
}

#[test]
fn module_report_empty_rows() {
    let report = ModuleReport {
        rows: vec![],
        total: Totals {
            code: 0,
            lines: 0,
            files: 0,
            bytes: 0,
            tokens: 0,
            avg_lines: 0,
        },
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
        top: 0,
    };
    let json = serde_json::to_string(&report).unwrap();
    let back: ModuleReport = serde_json::from_str(&json).unwrap();
    assert!(back.rows.is_empty());
}

#[test]
fn export_data_empty_rows() {
    let data = ExportData {
        rows: vec![],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::ParentsOnly,
    };
    let json = serde_json::to_string(&data).unwrap();
    let back: ExportData = serde_json::from_str(&json).unwrap();
    assert!(back.rows.is_empty());
}

// ---------------------------------------------------------------------------
// FileRow with special-character paths
// ---------------------------------------------------------------------------

#[test]
fn file_row_path_with_spaces() {
    let row = FileRow {
        path: "src/my module/hello world.rs".to_string(),
        module: "my module".to_string(),
        lang: "Rust".to_string(),
        kind: FileKind::Parent,
        code: 10,
        comments: 2,
        blanks: 1,
        lines: 13,
        bytes: 100,
        tokens: 25,
    };
    let json = serde_json::to_string(&row).unwrap();
    let back: FileRow = serde_json::from_str(&json).unwrap();
    assert_eq!(back.path, "src/my module/hello world.rs");
}

#[test]
fn file_row_path_with_unicode() {
    let row = FileRow {
        path: "src/日本語/ファイル.rs".to_string(),
        module: "日本語".to_string(),
        lang: "Rust".to_string(),
        kind: FileKind::Parent,
        code: 5,
        comments: 0,
        blanks: 0,
        lines: 5,
        bytes: 50,
        tokens: 12,
    };
    let json = serde_json::to_string(&row).unwrap();
    let back: FileRow = serde_json::from_str(&json).unwrap();
    assert_eq!(back.path, "src/日本語/ファイル.rs");
    assert_eq!(back.module, "日本語");
}

#[test]
fn file_row_path_with_dots() {
    let row = FileRow {
        path: "../outside/../tricky/./file.ext.bak".to_string(),
        module: "tricky".to_string(),
        lang: "Text".to_string(),
        kind: FileKind::Child,
        code: 0,
        comments: 0,
        blanks: 0,
        lines: 0,
        bytes: 0,
        tokens: 0,
    };
    let json = serde_json::to_string(&row).unwrap();
    let back: FileRow = serde_json::from_str(&json).unwrap();
    assert_eq!(back.path, "../outside/../tricky/./file.ext.bak");
}

// ---------------------------------------------------------------------------
// Enum variant roundtrips
// ---------------------------------------------------------------------------

#[test]
fn children_mode_roundtrip_all_variants() {
    for variant in [ChildrenMode::Collapse, ChildrenMode::Separate] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: ChildrenMode = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn child_include_mode_roundtrip_all_variants() {
    for variant in [ChildIncludeMode::Separate, ChildIncludeMode::ParentsOnly] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: ChildIncludeMode = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn table_format_roundtrip_all_variants() {
    for variant in [TableFormat::Md, TableFormat::Tsv, TableFormat::Json] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: TableFormat = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn export_format_roundtrip_all_variants() {
    for variant in [
        ExportFormat::Csv,
        ExportFormat::Jsonl,
        ExportFormat::Json,
        ExportFormat::Cyclonedx,
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: ExportFormat = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn config_mode_roundtrip_all_variants() {
    for variant in [ConfigMode::Auto, ConfigMode::None] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: ConfigMode = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn redact_mode_roundtrip_all_variants() {
    for variant in [RedactMode::None, RedactMode::Paths, RedactMode::All] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: RedactMode = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn file_kind_roundtrip_all_variants() {
    for variant in [FileKind::Parent, FileKind::Child] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: FileKind = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

// ---------------------------------------------------------------------------
// Multiple receipts don't interfere
// ---------------------------------------------------------------------------

#[test]
fn multiple_lang_rows_serialize_independently() {
    let row_a = LangRow {
        lang: "Rust".to_string(),
        code: 100,
        lines: 150,
        files: 5,
        bytes: 4000,
        tokens: 1000,
        avg_lines: 30,
    };
    let row_b = LangRow {
        lang: "Python".to_string(),
        code: 200,
        lines: 300,
        files: 10,
        bytes: 8000,
        tokens: 2000,
        avg_lines: 30,
    };
    let json_a = serde_json::to_string(&row_a).unwrap();
    let json_b = serde_json::to_string(&row_b).unwrap();
    let back_a: LangRow = serde_json::from_str(&json_a).unwrap();
    let back_b: LangRow = serde_json::from_str(&json_b).unwrap();
    assert_eq!(back_a.lang, "Rust");
    assert_eq!(back_b.lang, "Python");
    assert_eq!(back_a.code, 100);
    assert_eq!(back_b.code, 200);
}

#[test]
fn multiple_module_rows_serialize_independently() {
    let row_a = ModuleRow {
        module: "crates/foo".to_string(),
        code: 50,
        lines: 70,
        files: 3,
        bytes: 2000,
        tokens: 500,
        avg_lines: 23,
    };
    let row_b = ModuleRow {
        module: "crates/bar".to_string(),
        code: 80,
        lines: 100,
        files: 4,
        bytes: 3200,
        tokens: 800,
        avg_lines: 25,
    };
    let json_a = serde_json::to_string(&row_a).unwrap();
    let json_b = serde_json::to_string(&row_b).unwrap();
    let back_a: ModuleRow = serde_json::from_str(&json_a).unwrap();
    let back_b: ModuleRow = serde_json::from_str(&json_b).unwrap();
    assert_eq!(back_a, row_a);
    assert_eq!(back_b, row_b);
}
