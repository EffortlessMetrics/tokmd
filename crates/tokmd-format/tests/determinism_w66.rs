//! Determinism hardening tests for tokmd-format.
//!
//! Verifies that formatting the same data always produces byte-identical output
//! across Markdown, TSV, and JSON formats.

use proptest::prelude::*;
use tokmd_format::{compute_diff_rows, compute_diff_totals, render_diff_md};
use tokmd_types::*;

// -- Helpers --

fn sample_lang_row(lang: &str, code: usize) -> LangRow {
    LangRow {
        lang: lang.to_string(),
        code,
        lines: code + 50,
        files: 3,
        bytes: code * 4,
        tokens: code,
        avg_lines: if code > 0 { (code + 50) / 3 } else { 0 },
    }
}

fn sample_lang_report() -> LangReport {
    LangReport {
        rows: vec![
            sample_lang_row("Rust", 500),
            sample_lang_row("Python", 300),
            sample_lang_row("Go", 100),
        ],
        total: Totals {
            code: 900, lines: 1050, files: 9, bytes: 3600, tokens: 900, avg_lines: 116,
        },
        with_files: true,
        children: ChildrenMode::Collapse,
        top: 0,
    }
}

fn sample_lang_report_no_files() -> LangReport {
    LangReport {
        rows: vec![
            sample_lang_row("Rust", 500),
            sample_lang_row("Python", 300),
        ],
        total: Totals {
            code: 800, lines: 900, files: 6, bytes: 3200, tokens: 800, avg_lines: 150,
        },
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    }
}

fn sample_module_report() -> ModuleReport {
    ModuleReport {
        rows: vec![
            ModuleRow {
                module: "crates/tokmd".to_string(),
                code: 800, lines: 1100, files: 10, bytes: 3200, tokens: 800, avg_lines: 110,
            },
            ModuleRow {
                module: "src".to_string(),
                code: 200, lines: 300, files: 5, bytes: 800, tokens: 200, avg_lines: 60,
            },
        ],
        total: Totals {
            code: 1000, lines: 1400, files: 15, bytes: 4000, tokens: 1000, avg_lines: 93,
        },
        module_roots: vec!["crates".to_string()],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
        top: 0,
    }
}

fn sample_export_data() -> ExportData {
    ExportData {
        rows: vec![
            FileRow {
                path: "src/main.rs".to_string(),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code: 120, comments: 30, blanks: 20, lines: 170, bytes: 4800, tokens: 1200,
            },
            FileRow {
                path: "src/lib.rs".to_string(),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code: 80, comments: 10, blanks: 10, lines: 100, bytes: 3200, tokens: 800,
            },
        ],
        module_roots: vec!["crates".to_string()],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    }
}

fn default_scan_options() -> tokmd_settings::ScanOptions {
    tokmd_settings::ScanOptions::default()
}

fn lang_args_md() -> LangArgs {
    LangArgs {
        paths: vec![".".into()],
        format: TableFormat::Md,
        top: 0,
        files: true,
        children: ChildrenMode::Collapse,
    }
}

fn lang_args_tsv() -> LangArgs {
    LangArgs {
        paths: vec![".".into()],
        format: TableFormat::Tsv,
        top: 0,
        files: true,
        children: ChildrenMode::Collapse,
    }
}

fn module_args_md() -> ModuleArgs {
    ModuleArgs {
        paths: vec![".".into()],
        format: TableFormat::Md,
        top: 0,
        module_roots: vec!["crates".to_string()],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    }
}

fn module_args_tsv() -> ModuleArgs {
    ModuleArgs {
        paths: vec![".".into()],
        format: TableFormat::Tsv,
        top: 0,
        module_roots: vec!["crates".to_string()],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    }
}

fn export_args_csv() -> ExportArgs {
    ExportArgs {
        paths: vec![".".into()],
        format: ExportFormat::Csv,
        output: None,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
        min_code: 0,
        max_rows: 0,
        redact: RedactMode::None,
        meta: false,
        strip_prefix: None,
    }
}

fn export_args_jsonl() -> ExportArgs {
    ExportArgs {
        format: ExportFormat::Jsonl,
        meta: false,
        ..export_args_csv()
    }
}

// -- 1. Markdown lang report: same data -> same string every time --

#[test]
fn lang_md_output_is_byte_stable() {
    let report = sample_lang_report();
    let global = default_scan_options();
    let args = lang_args_md();
    let mut buf1 = Vec::new();
    tokmd_format::write_lang_report_to(&mut buf1, &report, &global, &args).unwrap();
    let mut buf2 = Vec::new();
    tokmd_format::write_lang_report_to(&mut buf2, &report, &global, &args).unwrap();
    assert_eq!(buf1, buf2, "Markdown lang output not byte-stable");
}

// -- 2. Markdown lang without files column --

#[test]
fn lang_md_no_files_output_is_byte_stable() {
    let report = sample_lang_report_no_files();
    let global = default_scan_options();
    let args = LangArgs {
        files: false,
        format: TableFormat::Md,
        ..lang_args_md()
    };
    let mut buf1 = Vec::new();
    tokmd_format::write_lang_report_to(&mut buf1, &report, &global, &args).unwrap();
    let mut buf2 = Vec::new();
    tokmd_format::write_lang_report_to(&mut buf2, &report, &global, &args).unwrap();
    assert_eq!(buf1, buf2);
}

// -- 3. TSV lang report: same bytes every time --

#[test]
fn lang_tsv_output_is_byte_stable() {
    let report = sample_lang_report();
    let global = default_scan_options();
    let args = lang_args_tsv();
    let mut buf1 = Vec::new();
    tokmd_format::write_lang_report_to(&mut buf1, &report, &global, &args).unwrap();
    let mut buf2 = Vec::new();
    tokmd_format::write_lang_report_to(&mut buf2, &report, &global, &args).unwrap();
    assert_eq!(buf1, buf2, "TSV lang output not byte-stable");
}

// -- 4. Module Markdown report stability --

#[test]
fn module_md_output_is_byte_stable() {
    let report = sample_module_report();
    let global = default_scan_options();
    let args = module_args_md();
    let mut buf1 = Vec::new();
    tokmd_format::write_module_report_to(&mut buf1, &report, &global, &args).unwrap();
    let mut buf2 = Vec::new();
    tokmd_format::write_module_report_to(&mut buf2, &report, &global, &args).unwrap();
    assert_eq!(buf1, buf2, "Markdown module output not byte-stable");
}

// -- 5. Module TSV report stability --

#[test]
fn module_tsv_output_is_byte_stable() {
    let report = sample_module_report();
    let global = default_scan_options();
    let args = module_args_tsv();
    let mut buf1 = Vec::new();
    tokmd_format::write_module_report_to(&mut buf1, &report, &global, &args).unwrap();
    let mut buf2 = Vec::new();
    tokmd_format::write_module_report_to(&mut buf2, &report, &global, &args).unwrap();
    assert_eq!(buf1, buf2, "TSV module output not byte-stable");
}

// -- 6. Export CSV stability --

#[test]
fn export_csv_output_is_byte_stable() {
    let data = sample_export_data();
    let args = export_args_csv();
    let mut buf1 = Vec::new();
    tokmd_format::write_export_csv_to(&mut buf1, &data, &args).unwrap();
    let mut buf2 = Vec::new();
    tokmd_format::write_export_csv_to(&mut buf2, &data, &args).unwrap();
    assert_eq!(buf1, buf2, "CSV export not byte-stable");
}

// -- 7. Export JSONL stability (no meta) --

#[test]
fn export_jsonl_no_meta_output_is_byte_stable() {
    let data = sample_export_data();
    let global = default_scan_options();
    let args = export_args_jsonl();
    let mut buf1 = Vec::new();
    tokmd_format::write_export_jsonl_to(&mut buf1, &data, &global, &args).unwrap();
    let mut buf2 = Vec::new();
    tokmd_format::write_export_jsonl_to(&mut buf2, &data, &global, &args).unwrap();
    assert_eq!(buf1, buf2, "JSONL export (no meta) not byte-stable");
}

// -- 8. compute_diff_rows is deterministic --

#[test]
fn diff_rows_are_deterministic() {
    let from = sample_lang_report();
    let to = LangReport {
        rows: vec![
            sample_lang_row("Rust", 600),
            sample_lang_row("Python", 350),
            sample_lang_row("TypeScript", 50),
        ],
        total: Totals {
            code: 1000, lines: 1150, files: 9, bytes: 4000, tokens: 1000, avg_lines: 127,
        },
        with_files: true,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let rows1 = compute_diff_rows(&from, &to);
    let rows2 = compute_diff_rows(&from, &to);
    let json1 = serde_json::to_string(&rows1).unwrap();
    let json2 = serde_json::to_string(&rows2).unwrap();
    assert_eq!(json1, json2, "diff rows not deterministic");
}

// -- 9. compute_diff_totals is deterministic --

#[test]
fn diff_totals_are_deterministic() {
    let rows = vec![
        DiffRow {
            lang: "Rust".into(),
            old_code: 100, new_code: 200, delta_code: 100,
            old_lines: 150, new_lines: 300, delta_lines: 150,
            old_files: 5, new_files: 8, delta_files: 3,
            old_bytes: 400, new_bytes: 800, delta_bytes: 400,
            old_tokens: 100, new_tokens: 200, delta_tokens: 100,
        },
        DiffRow {
            lang: "Go".into(),
            old_code: 50, new_code: 30, delta_code: -20,
            old_lines: 80, new_lines: 50, delta_lines: -30,
            old_files: 3, new_files: 2, delta_files: -1,
            old_bytes: 200, new_bytes: 120, delta_bytes: -80,
            old_tokens: 50, new_tokens: 30, delta_tokens: -20,
        },
    ];
    let t1 = compute_diff_totals(&rows);
    let t2 = compute_diff_totals(&rows);
    let json1 = serde_json::to_string(&t1).unwrap();
    let json2 = serde_json::to_string(&t2).unwrap();
    assert_eq!(json1, json2, "diff totals not deterministic");
}

// -- 10. render_diff_md is deterministic --

#[test]
fn diff_md_rendering_is_deterministic() {
    let rows = vec![
        DiffRow {
            lang: "Rust".into(),
            old_code: 100, new_code: 200, delta_code: 100,
            old_lines: 150, new_lines: 300, delta_lines: 150,
            old_files: 5, new_files: 8, delta_files: 3,
            old_bytes: 400, new_bytes: 800, delta_bytes: 400,
            old_tokens: 100, new_tokens: 200, delta_tokens: 100,
        },
    ];
    let totals = compute_diff_totals(&rows);
    let md1 = render_diff_md("v1.0", "v2.0", &rows, &totals);
    let md2 = render_diff_md("v1.0", "v2.0", &rows, &totals);
    assert_eq!(md1, md2, "diff markdown not deterministic");
}

// -- 11. Empty report rendering --

#[test]
fn empty_lang_report_md_is_byte_stable() {
    let report = LangReport {
        rows: vec![],
        total: Totals { code: 0, lines: 0, files: 0, bytes: 0, tokens: 0, avg_lines: 0 },
        with_files: true,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let global = default_scan_options();
    let args = lang_args_md();
    let mut buf1 = Vec::new();
    tokmd_format::write_lang_report_to(&mut buf1, &report, &global, &args).unwrap();
    let mut buf2 = Vec::new();
    tokmd_format::write_lang_report_to(&mut buf2, &report, &global, &args).unwrap();
    assert_eq!(buf1, buf2);
}

// -- 12. Single row report --

#[test]
fn single_row_lang_report_is_stable() {
    let report = LangReport {
        rows: vec![sample_lang_row("Rust", 1000)],
        total: Totals {
            code: 1000, lines: 1050, files: 3, bytes: 4000, tokens: 1000, avg_lines: 350,
        },
        with_files: true,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let global = default_scan_options();
    let args = lang_args_md();
    let mut buf1 = Vec::new();
    tokmd_format::write_lang_report_to(&mut buf1, &report, &global, &args).unwrap();
    let mut buf2 = Vec::new();
    tokmd_format::write_lang_report_to(&mut buf2, &report, &global, &args).unwrap();
    assert_eq!(buf1, buf2);
}

// -- 13. Diff with no changes produces no rows --

#[test]
fn diff_no_changes_produces_empty_deterministic() {
    let report = sample_lang_report();
    let rows1 = compute_diff_rows(&report, &report);
    let rows2 = compute_diff_rows(&report, &report);
    assert!(rows1.is_empty());
    assert_eq!(
        serde_json::to_string(&rows1).unwrap(),
        serde_json::to_string(&rows2).unwrap()
    );
}

// -- 14. CSV header order is deterministic --

#[test]
fn csv_header_order_is_deterministic() {
    let data = sample_export_data();
    let args = export_args_csv();
    let mut buf = Vec::new();
    tokmd_format::write_export_csv_to(&mut buf, &data, &args).unwrap();
    let csv_str = String::from_utf8(buf).unwrap();
    let header = csv_str.lines().next().unwrap();
    assert_eq!(header, "path,module,lang,kind,code,comments,blanks,lines,bytes,tokens");
}

// -- 15. Repeated formatting 100 times --

#[test]
fn repeated_md_formatting_100_times_is_stable() {
    let report = sample_lang_report();
    let global = default_scan_options();
    let args = lang_args_md();
    let outputs: Vec<Vec<u8>> = (0..100)
        .map(|_| {
            let mut buf = Vec::new();
            tokmd_format::write_lang_report_to(&mut buf, &report, &global, &args).unwrap();
            buf
        })
        .collect();
    assert!(outputs.windows(2).all(|w| w[0] == w[1]));
}

// -- 16. Markdown contains no HashMap-order artifacts --

#[test]
fn markdown_rows_maintain_input_order() {
    let report = LangReport {
        rows: vec![
            sample_lang_row("Rust", 500),
            sample_lang_row("Python", 300),
            sample_lang_row("Go", 100),
        ],
        total: Totals {
            code: 900, lines: 1050, files: 9, bytes: 3600, tokens: 900, avg_lines: 116,
        },
        with_files: true,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let global = default_scan_options();
    let args = lang_args_md();
    let mut buf = Vec::new();
    tokmd_format::write_lang_report_to(&mut buf, &report, &global, &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    let lines: Vec<&str> = output.lines().collect();
    assert!(lines.len() >= 6);
    assert!(lines[2].contains("Rust"));
    assert!(lines[3].contains("Python"));
    assert!(lines[4].contains("Go"));
}

// -- 17. TSV rows maintain input order --

#[test]
fn tsv_rows_maintain_input_order() {
    let report = sample_lang_report();
    let global = default_scan_options();
    let args = lang_args_tsv();
    let mut buf = Vec::new();
    tokmd_format::write_lang_report_to(&mut buf, &report, &global, &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    let lines: Vec<&str> = output.lines().collect();
    assert!(lines[1].starts_with("Rust"));
    assert!(lines[2].starts_with("Python"));
    assert!(lines[3].starts_with("Go"));
}

// -- 18. Module TSV row order stability --

#[test]
fn module_tsv_rows_maintain_input_order() {
    let report = sample_module_report();
    let global = default_scan_options();
    let args = module_args_tsv();
    let mut buf = Vec::new();
    tokmd_format::write_module_report_to(&mut buf, &report, &global, &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    let lines: Vec<&str> = output.lines().collect();
    assert!(lines[1].starts_with("crates/tokmd"));
    assert!(lines[2].starts_with("src"));
}

// -- Property tests --

proptest! {
    #[test]
    fn prop_lang_md_stability(
        code_a in 0usize..10_000,
        code_b in 0usize..10_000,
    ) {
        let report = LangReport {
            rows: vec![
                sample_lang_row("A", code_a),
                sample_lang_row("B", code_b),
            ],
            total: Totals {
                code: code_a + code_b,
                lines: code_a + code_b + 100,
                files: 6,
                bytes: (code_a + code_b) * 4,
                tokens: code_a + code_b,
                avg_lines: if (code_a + code_b + 100) > 0 { (code_a + code_b + 100) / 6 } else { 0 },
            },
            with_files: true,
            children: ChildrenMode::Collapse,
            top: 0,
        };
        let global = default_scan_options();
        let args = lang_args_md();
        let mut buf1 = Vec::new();
        tokmd_format::write_lang_report_to(&mut buf1, &report, &global, &args).unwrap();
        let mut buf2 = Vec::new();
        tokmd_format::write_lang_report_to(&mut buf2, &report, &global, &args).unwrap();
        prop_assert_eq!(buf1, buf2);
    }

    #[test]
    fn prop_diff_rows_stability(
        from_code in 0usize..10_000,
        to_code in 0usize..10_000,
    ) {
        let from = LangReport {
            rows: vec![sample_lang_row("Rust", from_code)],
            total: Totals {
                code: from_code, lines: from_code + 50, files: 3,
                bytes: from_code * 4, tokens: from_code, avg_lines: 0,
            },
            with_files: true,
            children: ChildrenMode::Collapse,
            top: 0,
        };
        let to = LangReport {
            rows: vec![sample_lang_row("Rust", to_code)],
            total: Totals {
                code: to_code, lines: to_code + 50, files: 3,
                bytes: to_code * 4, tokens: to_code, avg_lines: 0,
            },
            with_files: true,
            children: ChildrenMode::Collapse,
            top: 0,
        };
        let rows1 = compute_diff_rows(&from, &to);
        let rows2 = compute_diff_rows(&from, &to);
        let json1 = serde_json::to_string(&rows1).unwrap();
        let json2 = serde_json::to_string(&rows2).unwrap();
        prop_assert_eq!(json1, json2);
    }
}
