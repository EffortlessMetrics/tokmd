use tokmd_format::{write_lang_report_to, write_module_report_to};
use tokmd_settings::ScanOptions;
use tokmd_types::{
    ChildIncludeMode, ChildrenMode, LangArgs, LangReport, LangRow, ModuleArgs, ModuleReport,
    ModuleRow, TableFormat, Totals,
};

fn sample_lang_report() -> LangReport {
    LangReport {
        rows: vec![
            LangRow {
                lang: "Rust".to_string(),
                code: 5000,
                lines: 6200,
                files: 42,
                bytes: 180000,
                tokens: 12500,
                avg_lines: 148,
            },
            LangRow {
                lang: "Python".to_string(),
                code: 1200,
                lines: 1500,
                files: 8,
                bytes: 36000,
                tokens: 3000,
                avg_lines: 188,
            },
            LangRow {
                lang: "TOML".to_string(),
                code: 80,
                lines: 100,
                files: 3,
                bytes: 2400,
                tokens: 200,
                avg_lines: 33,
            },
        ],
        total: Totals {
            code: 6280,
            lines: 7800,
            files: 53,
            bytes: 218400,
            tokens: 15700,
            avg_lines: 147,
        },
        with_files: true,
        children: ChildrenMode::Collapse,
        top: 0,
    }
}

fn sample_module_report() -> ModuleReport {
    ModuleReport {
        rows: vec![
            ModuleRow {
                module: "crates/core".to_string(),
                code: 3000,
                lines: 3800,
                files: 25,
                bytes: 120000,
                tokens: 7500,
                avg_lines: 152,
            },
            ModuleRow {
                module: "crates/cli".to_string(),
                code: 2000,
                lines: 2400,
                files: 15,
                bytes: 60000,
                tokens: 5000,
                avg_lines: 160,
            },
            ModuleRow {
                module: "tests".to_string(),
                code: 1280,
                lines: 1600,
                files: 13,
                bytes: 38400,
                tokens: 3200,
                avg_lines: 123,
            },
        ],
        total: Totals {
            code: 6280,
            lines: 7800,
            files: 53,
            bytes: 218400,
            tokens: 15700,
            avg_lines: 147,
        },
        module_roots: vec!["crates".to_string()],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
        top: 0,
    }
}

// ── lang markdown ─────────────────────────────────────────────────────

#[test]
fn snapshot_lang_md_table() {
    let report = sample_lang_report();
    let args = LangArgs {
        paths: vec![".".into()],
        format: TableFormat::Md,
        top: 0,
        files: true,
        children: ChildrenMode::Collapse,
    };
    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, &report, &ScanOptions::default(), &args).unwrap();
    insta::assert_snapshot!(String::from_utf8(buf).unwrap());
}

// ── lang TSV ──────────────────────────────────────────────────────────

#[test]
fn snapshot_lang_tsv() {
    let report = sample_lang_report();
    let args = LangArgs {
        paths: vec![".".into()],
        format: TableFormat::Tsv,
        top: 0,
        files: true,
        children: ChildrenMode::Collapse,
    };
    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, &report, &ScanOptions::default(), &args).unwrap();
    insta::assert_snapshot!(String::from_utf8(buf).unwrap());
}

// ── lang JSON envelope ────────────────────────────────────────────────

#[test]
fn snapshot_lang_json_envelope() {
    let report = sample_lang_report();
    let args = LangArgs {
        paths: vec![".".into()],
        format: TableFormat::Json,
        top: 0,
        files: true,
        children: ChildrenMode::Collapse,
    };
    let mut buf = Vec::new();
    write_lang_report_to(&mut buf, &report, &ScanOptions::default(), &args).unwrap();
    let raw = String::from_utf8(buf).unwrap();
    // Re-parse and re-serialize to normalize dynamic fields (generated_at_ms, version)
    let mut v: serde_json::Value = serde_json::from_str(&raw).unwrap();
    v["generated_at_ms"] = serde_json::json!(0);
    v["tool"]["version"] = serde_json::json!("0.0.0-test");
    insta::assert_json_snapshot!(v);
}

// ── module breakdown ──────────────────────────────────────────────────

#[test]
fn snapshot_module_md_breakdown() {
    let report = sample_module_report();
    let args = ModuleArgs {
        paths: vec![".".into()],
        format: TableFormat::Md,
        top: 0,
        module_roots: vec!["crates".into()],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    };
    let mut buf = Vec::new();
    write_module_report_to(&mut buf, &report, &ScanOptions::default(), &args).unwrap();
    insta::assert_snapshot!(String::from_utf8(buf).unwrap());
}
