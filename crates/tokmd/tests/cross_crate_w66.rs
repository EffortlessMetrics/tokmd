//! Cross-crate integration tests (w66) exercising the pipeline:
//! types -> model -> format, verifying that types flow correctly between crates.

mod common;

use tempfile::TempDir;
use tokmd_model::{create_lang_report, create_module_report};
use tokmd_scan::scan;
use tokmd_settings::ScanOptions;
use tokmd_types::{
    ChildIncludeMode, ChildrenMode, ConfigMode, LangArgs, LangReceipt, LangReport, LangRow,
    ModuleArgs, ModuleReceipt, ModuleReport, ModuleRow, TableFormat, Totals, SCHEMA_VERSION,
};

fn make_project() -> TempDir {
    let dir = TempDir::new().expect("create tempdir");
    let root = dir.path();
    std::fs::create_dir_all(root.join(".git")).unwrap();
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(root.join("src/main.rs"), "fn main() {\n    println!(\"hello\");\n}\n").unwrap();
    std::fs::write(root.join("src/lib.rs"), "/// Doc\npub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n").unwrap();
    std::fs::create_dir_all(root.join("lib")).unwrap();
    std::fs::write(root.join("lib/util.py"), "# util\ndef greet(name):\n    return f\"Hello, {name}\"\n").unwrap();
    dir
}

fn opts() -> ScanOptions {
    ScanOptions { config: ConfigMode::None, no_ignore_vcs: true, ..Default::default() }
}

fn scan_lang(dir: &std::path::Path) -> LangReport {
    let langs = scan(&[dir.to_path_buf()], &opts()).expect("scan");
    create_lang_report(&langs, 0, true, ChildrenMode::Collapse)
}

fn scan_module(dir: &std::path::Path) -> ModuleReport {
    let langs = scan(&[dir.to_path_buf()], &opts()).expect("scan");
    create_module_report(&langs, &[], 1, ChildIncludeMode::Separate, 0)
}

fn synthetic_lang_report() -> LangReport {
    LangReport {
        rows: vec![
            LangRow { lang: "Rust".to_string(), code: 500, lines: 650, files: 10, bytes: 20_000, tokens: 5_000, avg_lines: 65 },
            LangRow { lang: "Python".to_string(), code: 300, lines: 400, files: 5, bytes: 12_000, tokens: 3_000, avg_lines: 80 },
        ],
        total: Totals { code: 800, lines: 1050, files: 15, bytes: 32_000, tokens: 8_000, avg_lines: 70 },
        with_files: true, children: ChildrenMode::Collapse, top: 0,
    }
}

fn synthetic_module_report() -> ModuleReport {
    ModuleReport {
        rows: vec![
            ModuleRow { module: "src".to_string(), code: 500, lines: 650, files: 10, bytes: 20_000, tokens: 5_000, avg_lines: 65 },
            ModuleRow { module: "lib".to_string(), code: 300, lines: 400, files: 5, bytes: 12_000, tokens: 3_000, avg_lines: 80 },
        ],
        total: Totals { code: 800, lines: 1050, files: 15, bytes: 32_000, tokens: 8_000, avg_lines: 70 },
        module_roots: vec![], module_depth: 1, children: ChildIncludeMode::Separate, top: 0,
    }
}

fn lang_args(dir: &std::path::Path, fmt: TableFormat) -> LangArgs {
    LangArgs { paths: vec![dir.to_path_buf()], format: fmt, top: 0, files: true, children: ChildrenMode::Collapse }
}

fn module_args(dir: &std::path::Path, fmt: TableFormat) -> ModuleArgs {
    ModuleArgs { paths: vec![dir.to_path_buf()], format: fmt, top: 0, module_roots: vec![], module_depth: 1, children: ChildIncludeMode::Separate }
}

// ===========================================================================
// 1. LangRow -> Markdown rendering
// ===========================================================================

#[test]
fn lang_to_md_has_table_header_and_total() {
    let proj = make_project();
    let report = scan_lang(proj.path());
    let args = lang_args(proj.path(), TableFormat::Md);
    let mut buf = Vec::new();
    tokmd_format::write_lang_report_to(&mut buf, &report, &opts(), &args).unwrap();
    let out = String::from_utf8(buf).unwrap();
    assert!(out.contains("|Lang|"), "must have Lang header");
    assert!(out.contains("|**Total**|"), "must have Total row");
}

#[test]
fn lang_md_includes_all_languages() {
    let proj = make_project();
    let report = scan_lang(proj.path());
    let args = lang_args(proj.path(), TableFormat::Md);
    let mut buf = Vec::new();
    tokmd_format::write_lang_report_to(&mut buf, &report, &opts(), &args).unwrap();
    let out = String::from_utf8(buf).unwrap();
    for row in &report.rows {
        assert!(out.contains(&row.lang), "MD should contain language {}", row.lang);
    }
}

#[test]
fn synthetic_lang_md_produces_valid_table() {
    let report = synthetic_lang_report();
    let args = LangArgs { paths: vec![".".into()], format: TableFormat::Md, top: 0, files: true, children: ChildrenMode::Collapse };
    let mut buf = Vec::new();
    tokmd_format::write_lang_report_to(&mut buf, &report, &opts(), &args).unwrap();
    let out = String::from_utf8(buf).unwrap();
    assert!(out.contains("|Rust|500|"));
    assert!(out.contains("|Python|300|"));
    assert!(out.contains("|**Total**|800|"));
}

// ===========================================================================
// 2. ModuleRow -> TSV rendering roundtrips
// ===========================================================================

#[test]
fn module_to_tsv_has_correct_columns() {
    let proj = make_project();
    let report = scan_module(proj.path());
    let args = module_args(proj.path(), TableFormat::Tsv);
    let mut buf = Vec::new();
    tokmd_format::write_module_report_to(&mut buf, &report, &opts(), &args).unwrap();
    let out = String::from_utf8(buf).unwrap();
    for line in out.lines().filter(|l| !l.is_empty()) {
        let cols: Vec<&str> = line.split('\t').collect();
        assert_eq!(cols.len(), 7, "TSV should have 7 columns: {line}");
    }
}

#[test]
fn module_tsv_header_is_correct() {
    let proj = make_project();
    let report = scan_module(proj.path());
    let args = module_args(proj.path(), TableFormat::Tsv);
    let mut buf = Vec::new();
    tokmd_format::write_module_report_to(&mut buf, &report, &opts(), &args).unwrap();
    let out = String::from_utf8(buf).unwrap();
    let header = out.lines().next().unwrap();
    assert_eq!(header, "Module\tCode\tLines\tFiles\tBytes\tTokens\tAvg");
}

#[test]
fn synthetic_module_tsv_roundtrip_values() {
    let report = synthetic_module_report();
    let args = ModuleArgs { paths: vec![".".into()], format: TableFormat::Tsv, top: 0, module_roots: vec![], module_depth: 1, children: ChildIncludeMode::Separate };
    let mut buf = Vec::new();
    tokmd_format::write_module_report_to(&mut buf, &report, &opts(), &args).unwrap();
    let out = String::from_utf8(buf).unwrap();
    let lines: Vec<&str> = out.lines().collect();
    assert!(lines.len() >= 4, "expected at least 4 TSV lines");
    let row1: Vec<&str> = lines[1].split('\t').collect();
    assert_eq!(row1[0], "src");
    assert_eq!(row1[1], "500");
    assert_eq!(row1[2], "650");
    let row2: Vec<&str> = lines[2].split('\t').collect();
    assert_eq!(row2[0], "lib");
    assert_eq!(row2[1], "300");
}

// ===========================================================================
// 3. Totals consistency across format outputs
// ===========================================================================

#[test]
fn totals_consistent_between_md_and_tsv_lang() {
    let proj = make_project();
    let report = scan_lang(proj.path());
    let md_args = lang_args(proj.path(), TableFormat::Md);
    let mut md_buf = Vec::new();
    tokmd_format::write_lang_report_to(&mut md_buf, &report, &opts(), &md_args).unwrap();
    let md_out = String::from_utf8(md_buf).unwrap();
    let tsv_args = lang_args(proj.path(), TableFormat::Tsv);
    let mut tsv_buf = Vec::new();
    tokmd_format::write_lang_report_to(&mut tsv_buf, &report, &opts(), &tsv_args).unwrap();
    let tsv_out = String::from_utf8(tsv_buf).unwrap();
    let total_code = report.total.code.to_string();
    assert!(md_out.contains(&total_code), "MD must contain total code {total_code}");
    assert!(tsv_out.contains(&total_code), "TSV must contain total code {total_code}");
}

#[test]
fn totals_consistent_between_md_and_tsv_module() {
    let proj = make_project();
    let report = scan_module(proj.path());
    let md_args = module_args(proj.path(), TableFormat::Md);
    let mut md_buf = Vec::new();
    tokmd_format::write_module_report_to(&mut md_buf, &report, &opts(), &md_args).unwrap();
    let md_out = String::from_utf8(md_buf).unwrap();
    let tsv_args = module_args(proj.path(), TableFormat::Tsv);
    let mut tsv_buf = Vec::new();
    tokmd_format::write_module_report_to(&mut tsv_buf, &report, &opts(), &tsv_args).unwrap();
    let tsv_out = String::from_utf8(tsv_buf).unwrap();
    let total_code = report.total.code.to_string();
    assert!(md_out.contains(&total_code));
    assert!(tsv_out.contains(&total_code));
}

#[test]
fn totals_consistent_between_md_and_json_lang() {
    let proj = make_project();
    let report = scan_lang(proj.path());
    let json_args = lang_args(proj.path(), TableFormat::Json);
    let mut json_buf = Vec::new();
    tokmd_format::write_lang_report_to(&mut json_buf, &report, &opts(), &json_args).unwrap();
    let receipt: LangReceipt = serde_json::from_slice(&json_buf).unwrap();
    assert_eq!(receipt.report.total.code, report.total.code);
    assert_eq!(receipt.report.total.lines, report.total.lines);
    assert_eq!(receipt.report.total.files, report.total.files);
}

#[test]
fn totals_consistent_between_md_and_json_module() {
    let proj = make_project();
    let report = scan_module(proj.path());
    let json_args = module_args(proj.path(), TableFormat::Json);
    let mut json_buf = Vec::new();
    tokmd_format::write_module_report_to(&mut json_buf, &report, &opts(), &json_args).unwrap();
    let receipt: ModuleReceipt = serde_json::from_slice(&json_buf).unwrap();
    assert_eq!(receipt.report.total.code, report.total.code);
    assert_eq!(receipt.report.total.lines, report.total.lines);
    assert_eq!(receipt.report.total.files, report.total.files);
}

// ===========================================================================
// 4. Schema version consistency
// ===========================================================================

#[test]
fn lang_json_has_correct_schema_version() {
    let proj = make_project();
    let report = scan_lang(proj.path());
    let args = lang_args(proj.path(), TableFormat::Json);
    let mut buf = Vec::new();
    tokmd_format::write_lang_report_to(&mut buf, &report, &opts(), &args).unwrap();
    let receipt: LangReceipt = serde_json::from_slice(&buf).unwrap();
    assert_eq!(receipt.schema_version, SCHEMA_VERSION);
}

#[test]
fn module_json_has_correct_schema_version() {
    let proj = make_project();
    let report = scan_module(proj.path());
    let args = module_args(proj.path(), TableFormat::Json);
    let mut buf = Vec::new();
    tokmd_format::write_module_report_to(&mut buf, &report, &opts(), &args).unwrap();
    let receipt: ModuleReceipt = serde_json::from_slice(&buf).unwrap();
    assert_eq!(receipt.schema_version, SCHEMA_VERSION);
}

// ===========================================================================
// 5. Synthetic data: Totals sum matches row sums
// ===========================================================================

#[test]
fn synthetic_lang_totals_match_row_sums() {
    let report = synthetic_lang_report();
    let sum_code: usize = report.rows.iter().map(|r| r.code).sum();
    let sum_lines: usize = report.rows.iter().map(|r| r.lines).sum();
    let sum_files: usize = report.rows.iter().map(|r| r.files).sum();
    assert_eq!(report.total.code, sum_code);
    assert_eq!(report.total.lines, sum_lines);
    assert_eq!(report.total.files, sum_files);
}

#[test]
fn synthetic_module_totals_match_row_sums() {
    let report = synthetic_module_report();
    let sum_code: usize = report.rows.iter().map(|r| r.code).sum();
    let sum_lines: usize = report.rows.iter().map(|r| r.lines).sum();
    let sum_files: usize = report.rows.iter().map(|r| r.files).sum();
    assert_eq!(report.total.code, sum_code);
    assert_eq!(report.total.lines, sum_lines);
    assert_eq!(report.total.files, sum_files);
}

// ===========================================================================
// 6. Row data flows correctly through format pipeline
// ===========================================================================

#[test]
fn lang_json_rows_match_original_report() {
    let proj = make_project();
    let report = scan_lang(proj.path());
    let args = lang_args(proj.path(), TableFormat::Json);
    let mut buf = Vec::new();
    tokmd_format::write_lang_report_to(&mut buf, &report, &opts(), &args).unwrap();
    let receipt: LangReceipt = serde_json::from_slice(&buf).unwrap();
    assert_eq!(receipt.report.rows.len(), report.rows.len());
    for (orig, parsed) in report.rows.iter().zip(receipt.report.rows.iter()) {
        assert_eq!(orig.lang, parsed.lang);
        assert_eq!(orig.code, parsed.code);
        assert_eq!(orig.lines, parsed.lines);
        assert_eq!(orig.files, parsed.files);
    }
}

#[test]
fn module_json_rows_match_original_report() {
    let proj = make_project();
    let report = scan_module(proj.path());
    let args = module_args(proj.path(), TableFormat::Json);
    let mut buf = Vec::new();
    tokmd_format::write_module_report_to(&mut buf, &report, &opts(), &args).unwrap();
    let receipt: ModuleReceipt = serde_json::from_slice(&buf).unwrap();
    assert_eq!(receipt.report.rows.len(), report.rows.len());
    for (orig, parsed) in report.rows.iter().zip(receipt.report.rows.iter()) {
        assert_eq!(orig.module, parsed.module);
        assert_eq!(orig.code, parsed.code);
    }
}

// ===========================================================================
// 7. Scan produces non-empty data for known project
// ===========================================================================

#[test]
fn scan_produces_nonempty_lang_report() {
    let proj = make_project();
    let report = scan_lang(proj.path());
    assert!(!report.rows.is_empty(), "lang report should have rows");
    assert!(report.total.code > 0, "total code should be positive");
}

#[test]
fn scan_produces_nonempty_module_report() {
    let proj = make_project();
    let report = scan_module(proj.path());
    assert!(!report.rows.is_empty(), "module report should have rows");
    assert!(report.total.code > 0, "total code should be positive");
}

// ===========================================================================
// 8. MD and TSV outputs are non-empty
// ===========================================================================

#[test]
fn lang_md_output_is_nonempty() {
    let proj = make_project();
    let report = scan_lang(proj.path());
    let args = lang_args(proj.path(), TableFormat::Md);
    let mut buf = Vec::new();
    tokmd_format::write_lang_report_to(&mut buf, &report, &opts(), &args).unwrap();
    assert!(!buf.is_empty());
}

#[test]
fn module_tsv_output_is_nonempty() {
    let proj = make_project();
    let report = scan_module(proj.path());
    let args = module_args(proj.path(), TableFormat::Tsv);
    let mut buf = Vec::new();
    tokmd_format::write_module_report_to(&mut buf, &report, &opts(), &args).unwrap();
    assert!(!buf.is_empty());
}