//! # tokmd-format
//!
//! **Tier 2 (Formatting)**
//!
//! This crate handles the rendering and serialization of `tokmd` receipts.
//! It supports Markdown, TSV, JSON, JSONL, CSV, and CycloneDX formats.
//!
//! ## What belongs here
//! * Serialization logic (JSON/CSV/CycloneDX)
//! * Markdown and TSV table rendering
//! * Output file writing
//! * Redaction integration (via internal `redact` module)
//! * ScanArgs integration (via internal `scan_args` module)
//! * Analysis receipt rendering under [`analysis`]
//!
//! ## What does NOT belong here
//! * Business logic (calculating stats)
//! * CLI argument parsing
//! * Analysis computation (use tokmd-analysis)

use std::fmt::Write as FmtWrite;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(test)]
use std::path::PathBuf;

use anyhow::Result;

use tokmd_settings::ScanOptions;
use tokmd_types::{
    LangArgs, LangArgsMeta, LangReceipt, LangReport, ModuleArgs, ModuleArgsMeta, ModuleReceipt,
    ModuleReport, RedactMode, ScanArgs, ScanStatus, TableFormat, ToolInfo,
};

pub mod analysis;
pub mod badge;
mod diff;
mod export;
pub mod export_tree;
#[cfg(feature = "fun")]
pub mod fun;
pub mod redact;
pub mod scan_args;

pub use badge::badge_svg;
pub use diff::{
    DiffColorMode, DiffRenderOptions, compute_diff_rows, compute_diff_totals, create_diff_receipt,
    render_diff_md, render_diff_md_with_options,
};
pub use export::{
    write_export, write_export_csv_to, write_export_cyclonedx_to,
    write_export_cyclonedx_with_options, write_export_json_to, write_export_jsonl_to,
    write_export_jsonl_to_file,
};
pub use export_tree::{render_analysis_tree, render_handoff_tree};
pub use redact::{redact_path, short_hash};
pub use scan_args::{normalize_scan_input, scan_args};

fn redact_module_roots(roots: &[String], redact: RedactMode) -> Vec<String> {
    if redact == RedactMode::All {
        roots.iter().map(|r| short_hash(r)).collect()
    } else {
        roots.to_vec()
    }
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

// -----------------------
// Language summary output
// -----------------------

/// Write a language report to a writer.
///
/// This is the core implementation that can be tested with any `Write` sink.
pub fn write_lang_report_to<W: Write>(
    mut out: W,
    report: &LangReport,
    global: &ScanOptions,
    args: &LangArgs,
) -> Result<()> {
    match args.format {
        TableFormat::Md => {
            out.write_all(render_lang_md(report).as_bytes())?;
        }
        TableFormat::Tsv => {
            out.write_all(render_lang_tsv(report).as_bytes())?;
        }
        TableFormat::Json => {
            let receipt = LangReceipt {
                schema_version: tokmd_types::SCHEMA_VERSION,
                generated_at_ms: now_ms(),
                tool: ToolInfo::current(),
                mode: "lang".to_string(),
                status: ScanStatus::Complete,
                warnings: vec![],
                scan: scan_args(&args.paths, global, None),
                args: LangArgsMeta {
                    format: "json".to_string(),
                    top: report.top,
                    with_files: report.with_files,
                    children: report.children,
                },
                report: report.clone(),
            };
            writeln!(out, "{}", serde_json::to_string(&receipt)?)?;
        }
    }
    Ok(())
}

/// Print a language report to stdout.
///
/// Thin wrapper around [`write_lang_report_to`] for stdout.
pub fn print_lang_report(report: &LangReport, global: &ScanOptions, args: &LangArgs) -> Result<()> {
    let stdout = io::stdout();
    let out = stdout.lock();
    write_lang_report_to(out, report, global, args)
}

fn render_lang_md(report: &LangReport) -> String {
    // Heuristic: (rows + 3) * 80 chars per row
    let mut s = String::with_capacity((report.rows.len() + 3) * 80);

    if report.with_files {
        s.push_str("|Lang|Code|Lines|Files|Bytes|Tokens|Avg|\n");
        s.push_str("|---|---:|---:|---:|---:|---:|---:|\n");
        for r in &report.rows {
            let _ = writeln!(
                s,
                "|{}|{}|{}|{}|{}|{}|{}|",
                r.lang, r.code, r.lines, r.files, r.bytes, r.tokens, r.avg_lines
            );
        }
        let _ = writeln!(
            s,
            "|**Total**|{}|{}|{}|{}|{}|{}|",
            report.total.code,
            report.total.lines,
            report.total.files,
            report.total.bytes,
            report.total.tokens,
            report.total.avg_lines
        );
    } else {
        s.push_str("|Lang|Code|Lines|Bytes|Tokens|\n");
        s.push_str("|---|---:|---:|---:|---:|\n");
        for r in &report.rows {
            let _ = writeln!(
                s,
                "|{}|{}|{}|{}|{}|",
                r.lang, r.code, r.lines, r.bytes, r.tokens
            );
        }
        let _ = writeln!(
            s,
            "|**Total**|{}|{}|{}|{}|",
            report.total.code, report.total.lines, report.total.bytes, report.total.tokens
        );
    }

    s
}

fn render_lang_tsv(report: &LangReport) -> String {
    // Heuristic: (rows + 2) * 64 chars per row
    let mut s = String::with_capacity((report.rows.len() + 2) * 64);

    if report.with_files {
        s.push_str("Lang\tCode\tLines\tFiles\tBytes\tTokens\tAvg\n");
        for r in &report.rows {
            let _ = writeln!(
                s,
                "{}\t{}\t{}\t{}\t{}\t{}\t{}",
                r.lang, r.code, r.lines, r.files, r.bytes, r.tokens, r.avg_lines
            );
        }
        let _ = writeln!(
            s,
            "Total\t{}\t{}\t{}\t{}\t{}\t{}",
            report.total.code,
            report.total.lines,
            report.total.files,
            report.total.bytes,
            report.total.tokens,
            report.total.avg_lines
        );
    } else {
        s.push_str("Lang\tCode\tLines\tBytes\tTokens\n");
        for r in &report.rows {
            let _ = writeln!(
                s,
                "{}\t{}\t{}\t{}\t{}",
                r.lang, r.code, r.lines, r.bytes, r.tokens
            );
        }
        let _ = writeln!(
            s,
            "Total\t{}\t{}\t{}\t{}",
            report.total.code, report.total.lines, report.total.bytes, report.total.tokens
        );
    }

    s
}

// ---------------------
// Module summary output
// ---------------------

/// Write a module report to a writer.
///
/// This is the core implementation that can be tested with any `Write` sink.
pub fn write_module_report_to<W: Write>(
    mut out: W,
    report: &ModuleReport,
    global: &ScanOptions,
    args: &ModuleArgs,
) -> Result<()> {
    match args.format {
        TableFormat::Md => {
            out.write_all(render_module_md(report).as_bytes())?;
        }
        TableFormat::Tsv => {
            out.write_all(render_module_tsv(report).as_bytes())?;
        }
        TableFormat::Json => {
            let receipt = ModuleReceipt {
                schema_version: tokmd_types::SCHEMA_VERSION,
                generated_at_ms: now_ms(),
                tool: ToolInfo::current(),
                mode: "module".to_string(),
                status: ScanStatus::Complete,
                warnings: vec![],
                scan: scan_args(&args.paths, global, None),
                args: ModuleArgsMeta {
                    format: "json".to_string(),
                    top: report.top,
                    module_roots: report.module_roots.clone(),
                    module_depth: report.module_depth,
                    children: report.children,
                },
                report: report.clone(),
            };
            writeln!(out, "{}", serde_json::to_string(&receipt)?)?;
        }
    }
    Ok(())
}

/// Print a module report to stdout.
///
/// Thin wrapper around [`write_module_report_to`] for stdout.
pub fn print_module_report(
    report: &ModuleReport,
    global: &ScanOptions,
    args: &ModuleArgs,
) -> Result<()> {
    let stdout = io::stdout();
    let out = stdout.lock();
    write_module_report_to(out, report, global, args)
}

fn render_module_md(report: &ModuleReport) -> String {
    // Heuristic: (rows + 3) * 80 chars per row
    let mut s = String::with_capacity((report.rows.len() + 3) * 80);
    s.push_str("|Module|Code|Lines|Files|Bytes|Tokens|Avg|\n");
    s.push_str("|---|---:|---:|---:|---:|---:|---:|\n");
    for r in &report.rows {
        let _ = writeln!(
            s,
            "|{}|{}|{}|{}|{}|{}|{}|",
            r.module, r.code, r.lines, r.files, r.bytes, r.tokens, r.avg_lines
        );
    }
    let _ = writeln!(
        s,
        "|**Total**|{}|{}|{}|{}|{}|{}|",
        report.total.code,
        report.total.lines,
        report.total.files,
        report.total.bytes,
        report.total.tokens,
        report.total.avg_lines
    );
    s
}

fn render_module_tsv(report: &ModuleReport) -> String {
    // Heuristic: (rows + 2) * 64 chars per row
    let mut s = String::with_capacity((report.rows.len() + 2) * 64);
    s.push_str("Module\tCode\tLines\tFiles\tBytes\tTokens\tAvg\n");
    for r in &report.rows {
        let _ = writeln!(
            s,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}",
            r.module, r.code, r.lines, r.files, r.bytes, r.tokens, r.avg_lines
        );
    }
    let _ = writeln!(
        s,
        "Total\t{}\t{}\t{}\t{}\t{}\t{}",
        report.total.code,
        report.total.lines,
        report.total.files,
        report.total.bytes,
        report.total.tokens,
        report.total.avg_lines
    );
    s
}

// -----------------
// Run command helpers
// -----------------

/// Write a lang report as JSON to a file path.
///
/// This is a convenience function for the `run` command that accepts
/// pre-constructed `ScanArgs` and `LangArgsMeta` rather than requiring
/// the full CLI args structs.
pub fn write_lang_json_to_file(
    path: &Path,
    report: &LangReport,
    scan: &ScanArgs,
    args_meta: &LangArgsMeta,
) -> Result<()> {
    let receipt = LangReceipt {
        schema_version: tokmd_types::SCHEMA_VERSION,
        generated_at_ms: now_ms(),
        tool: ToolInfo::current(),
        mode: "lang".to_string(),
        status: ScanStatus::Complete,
        warnings: vec![],
        scan: scan.clone(),
        args: args_meta.clone(),
        report: report.clone(),
    };
    let file = File::create(path)?;
    serde_json::to_writer(file, &receipt)?;
    Ok(())
}

/// Write a module report as JSON to a file path.
///
/// This is a convenience function for the `run` command that accepts
/// pre-constructed `ScanArgs` and `ModuleArgsMeta` rather than requiring
/// the full CLI args structs.
pub fn write_module_json_to_file(
    path: &Path,
    report: &ModuleReport,
    scan: &ScanArgs,
    args_meta: &ModuleArgsMeta,
    redact: RedactMode,
) -> Result<()> {
    let mut final_args = args_meta.clone();
    let mut final_report = report.clone();

    if redact == RedactMode::All {
        final_args.module_roots = redact_module_roots(&final_args.module_roots, redact);
        final_report.module_roots = redact_module_roots(&final_report.module_roots, redact);
        for row in &mut final_report.rows {
            row.module = short_hash(&row.module);
        }
    }

    let receipt = ModuleReceipt {
        schema_version: tokmd_types::SCHEMA_VERSION,
        generated_at_ms: now_ms(),
        tool: ToolInfo::current(),
        mode: "module".to_string(),
        status: ScanStatus::Complete,
        warnings: vec![],
        scan: scan.clone(),
        args: final_args,
        report: final_report,
    };
    let file = File::create(path)?;
    serde_json::to_writer(file, &receipt)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use tokmd_settings::ChildrenMode;
    use tokmd_types::{LangRow, ModuleRow, Totals};

    fn sample_lang_report(with_files: bool) -> LangReport {
        LangReport {
            rows: vec![
                LangRow {
                    lang: "Rust".to_string(),
                    code: 1000,
                    lines: 1200,
                    files: 10,
                    bytes: 50000,
                    tokens: 2500,
                    avg_lines: 120,
                },
                LangRow {
                    lang: "TOML".to_string(),
                    code: 50,
                    lines: 60,
                    files: 2,
                    bytes: 1000,
                    tokens: 125,
                    avg_lines: 30,
                },
            ],
            total: Totals {
                code: 1050,
                lines: 1260,
                files: 12,
                bytes: 51000,
                tokens: 2625,
                avg_lines: 105,
            },
            with_files,
            children: ChildrenMode::Collapse,
            top: 0,
        }
    }

    fn sample_module_report() -> ModuleReport {
        ModuleReport {
            rows: vec![
                ModuleRow {
                    module: "crates/foo".to_string(),
                    code: 800,
                    lines: 950,
                    files: 8,
                    bytes: 40000,
                    tokens: 2000,
                    avg_lines: 119,
                },
                ModuleRow {
                    module: "crates/bar".to_string(),
                    code: 200,
                    lines: 250,
                    files: 2,
                    bytes: 10000,
                    tokens: 500,
                    avg_lines: 125,
                },
            ],
            total: Totals {
                code: 1000,
                lines: 1200,
                files: 10,
                bytes: 50000,
                tokens: 2500,
                avg_lines: 120,
            },
            module_roots: vec!["crates".to_string()],
            module_depth: 2,
            children: tokmd_settings::ChildIncludeMode::Separate,
            top: 0,
        }
    }

    // ========================
    // Language Markdown Render Tests
    // ========================

    #[test]
    fn render_lang_md_without_files() {
        let report = sample_lang_report(false);
        let output = render_lang_md(&report);

        // Check header
        assert!(output.contains("|Lang|Code|Lines|Bytes|Tokens|"));
        // Check no Files/Avg columns
        assert!(!output.contains("|Files|"));
        assert!(!output.contains("|Avg|"));
        // Check row data
        assert!(output.contains("|Rust|1000|1200|50000|2500|"));
        assert!(output.contains("|TOML|50|60|1000|125|"));
        // Check total
        assert!(output.contains("|**Total**|1050|1260|51000|2625|"));
    }

    #[test]
    fn render_lang_md_with_files() {
        let report = sample_lang_report(true);
        let output = render_lang_md(&report);

        // Check header includes Files and Avg
        assert!(output.contains("|Lang|Code|Lines|Files|Bytes|Tokens|Avg|"));
        // Check row data includes file counts
        assert!(output.contains("|Rust|1000|1200|10|50000|2500|120|"));
        assert!(output.contains("|TOML|50|60|2|1000|125|30|"));
        // Check total
        assert!(output.contains("|**Total**|1050|1260|12|51000|2625|105|"));
    }

    #[test]
    fn render_lang_md_table_structure() {
        let report = sample_lang_report(true);
        let output = render_lang_md(&report);

        // Verify markdown table structure
        let lines: Vec<&str> = output.lines().collect();
        assert!(lines.len() >= 4); // header, separator, 2 data rows, total

        // Check separator line
        assert!(lines[1].contains("|---|"));
        assert!(lines[1].contains(":")); // Right-aligned columns
    }

    // ========================
    // Language TSV Render Tests
    // ========================

    #[test]
    fn render_lang_tsv_without_files() {
        let report = sample_lang_report(false);
        let output = render_lang_tsv(&report);

        // Check header
        assert!(output.starts_with("Lang\tCode\tLines\tBytes\tTokens\n"));
        // Check no Files/Avg columns
        assert!(!output.contains("\tFiles\t"));
        assert!(!output.contains("\tAvg"));
        // Check row data
        assert!(output.contains("Rust\t1000\t1200\t50000\t2500"));
        assert!(output.contains("TOML\t50\t60\t1000\t125"));
        // Check total
        assert!(output.contains("Total\t1050\t1260\t51000\t2625"));
    }

    #[test]
    fn render_lang_tsv_with_files() {
        let report = sample_lang_report(true);
        let output = render_lang_tsv(&report);

        // Check header includes Files and Avg
        assert!(output.starts_with("Lang\tCode\tLines\tFiles\tBytes\tTokens\tAvg\n"));
        // Check row data includes file counts
        assert!(output.contains("Rust\t1000\t1200\t10\t50000\t2500\t120"));
        assert!(output.contains("TOML\t50\t60\t2\t1000\t125\t30"));
    }

    #[test]
    fn render_lang_tsv_tab_separated() {
        let report = sample_lang_report(false);
        let output = render_lang_tsv(&report);

        // Each data line should have exactly 4 tabs (5 columns)
        for line in output.lines().skip(1) {
            // Skip header
            if line.starts_with("Total") || line.starts_with("Rust") || line.starts_with("TOML") {
                assert_eq!(line.matches('\t').count(), 4);
            }
        }
    }

    // ========================
    // Module Markdown Render Tests
    // ========================

    #[test]
    fn render_module_md_structure() {
        let report = sample_module_report();
        let output = render_module_md(&report);

        // Check header
        assert!(output.contains("|Module|Code|Lines|Files|Bytes|Tokens|Avg|"));
        // Check module data
        assert!(output.contains("|crates/foo|800|950|8|40000|2000|119|"));
        assert!(output.contains("|crates/bar|200|250|2|10000|500|125|"));
        // Check total
        assert!(output.contains("|**Total**|1000|1200|10|50000|2500|120|"));
    }

    #[test]
    fn render_module_md_table_format() {
        let report = sample_module_report();
        let output = render_module_md(&report);

        let lines: Vec<&str> = output.lines().collect();
        // Header, separator, 2 rows, total
        assert_eq!(lines.len(), 5);
        // Separator has right-alignment markers
        assert!(lines[1].contains("---:"));
    }

    // ========================
    // Module TSV Render Tests
    // ========================

    #[test]
    fn render_module_tsv_structure() {
        let report = sample_module_report();
        let output = render_module_tsv(&report);

        // Check header
        assert!(output.starts_with("Module\tCode\tLines\tFiles\tBytes\tTokens\tAvg\n"));
        // Check data
        assert!(output.contains("crates/foo\t800\t950\t8\t40000\t2000\t119"));
        assert!(output.contains("crates/bar\t200\t250\t2\t10000\t500\t125"));
        // Check total
        assert!(output.contains("Total\t1000\t1200\t10\t50000\t2500\t120"));
    }

    #[test]
    fn render_module_tsv_tab_count() {
        let report = sample_module_report();
        let output = render_module_tsv(&report);

        // Each data line should have exactly 6 tabs (7 columns)
        for line in output.lines() {
            assert_eq!(line.matches('\t').count(), 6);
        }
    }

    // ========================
    // Path Normalization Tests
    // ========================

    #[test]
    fn normalize_scan_input_forward_slash() {
        let p = Path::new("src/lib.rs");
        let normalized = normalize_scan_input(p);
        assert_eq!(normalized, "src/lib.rs");
    }

    #[test]
    fn normalize_scan_input_backslash_to_forward() {
        let p = Path::new("src\\lib.rs");
        let normalized = normalize_scan_input(p);
        assert_eq!(normalized, "src/lib.rs");
    }

    #[test]
    fn normalize_scan_input_strips_dot_slash() {
        let p = Path::new("./src/lib.rs");
        let normalized = normalize_scan_input(p);
        assert_eq!(normalized, "src/lib.rs");
    }

    #[test]
    fn normalize_scan_input_current_dir() {
        let p = Path::new(".");
        let normalized = normalize_scan_input(p);
        assert_eq!(normalized, ".");
    }

    // ========================
    // Property-Based Tests
    // ========================

    proptest! {
        #[test]
        fn normalize_scan_input_no_backslash(s in "[a-zA-Z0-9_/\\\\.]+") {
            let p = Path::new(&s);
            let normalized = normalize_scan_input(p);
            prop_assert!(!normalized.contains('\\'), "Should not contain backslash: {}", normalized);
        }

        #[test]
        fn normalize_scan_input_no_leading_dot_slash(s in "[a-zA-Z0-9_/\\\\.]+") {
            let p = Path::new(&s);
            let normalized = normalize_scan_input(p);
            prop_assert!(!normalized.starts_with("./"), "Should not start with ./: {}", normalized);
        }

    }

    // ========================
    // Snapshot Tests
    // ========================

    #[test]
    fn snapshot_lang_md_with_files() {
        let report = sample_lang_report(true);
        let output = render_lang_md(&report);
        insta::assert_snapshot!(output);
    }

    #[test]
    fn snapshot_lang_md_without_files() {
        let report = sample_lang_report(false);
        let output = render_lang_md(&report);
        insta::assert_snapshot!(output);
    }

    #[test]
    fn snapshot_lang_tsv_with_files() {
        let report = sample_lang_report(true);
        let output = render_lang_tsv(&report);
        insta::assert_snapshot!(output);
    }

    #[test]
    fn snapshot_module_md() {
        let report = sample_module_report();
        let output = render_module_md(&report);
        insta::assert_snapshot!(output);
    }

    #[test]
    fn snapshot_module_tsv() {
        let report = sample_module_report();
        let output = render_module_tsv(&report);
        insta::assert_snapshot!(output);
    }

    // ========================
    // Diff Render Tests
    // ========================

    // ========================
    // write_*_to Tests (mutation killers)
    // ========================

    fn sample_global_args() -> ScanOptions {
        ScanOptions::default()
    }

    fn sample_lang_args(format: TableFormat) -> LangArgs {
        LangArgs {
            paths: vec![PathBuf::from(".")],
            format,
            top: 0,
            files: false,
            children: ChildrenMode::Collapse,
        }
    }

    fn sample_module_args(format: TableFormat) -> ModuleArgs {
        ModuleArgs {
            paths: vec![PathBuf::from(".")],
            format,
            top: 0,
            module_roots: vec!["crates".to_string()],
            module_depth: 2,
            children: tokmd_settings::ChildIncludeMode::Separate,
        }
    }

    #[test]
    fn write_lang_report_to_md_writes_content() {
        let report = sample_lang_report(true);
        let global = sample_global_args();
        let args = sample_lang_args(TableFormat::Md);
        let mut buf = Vec::new();

        write_lang_report_to(&mut buf, &report, &global, &args).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(!output.is_empty(), "output must not be empty");
        assert!(output.contains("|Lang|"), "must contain markdown header");
        assert!(output.contains("|Rust|"), "must contain Rust row");
        assert!(output.contains("|**Total**|"), "must contain total row");
    }

    #[test]
    fn write_lang_report_to_tsv_writes_content() {
        let report = sample_lang_report(false);
        let global = sample_global_args();
        let args = sample_lang_args(TableFormat::Tsv);
        let mut buf = Vec::new();

        write_lang_report_to(&mut buf, &report, &global, &args).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(!output.is_empty(), "output must not be empty");
        assert!(output.contains("Lang\t"), "must contain TSV header");
        assert!(output.contains("Rust\t"), "must contain Rust row");
        assert!(output.contains("Total\t"), "must contain total row");
    }

    #[test]
    fn write_lang_report_to_json_writes_receipt() {
        let report = sample_lang_report(true);
        let global = sample_global_args();
        let args = sample_lang_args(TableFormat::Json);
        let mut buf = Vec::new();

        write_lang_report_to(&mut buf, &report, &global, &args).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(!output.is_empty(), "output must not be empty");
        // Parse as JSON to verify valid receipt
        let receipt: LangReceipt = serde_json::from_str(&output).unwrap();
        assert_eq!(receipt.mode, "lang");
        assert_eq!(receipt.report.rows.len(), 2);
        assert_eq!(receipt.report.total.code, 1050);
    }

    #[test]
    fn write_module_report_to_md_writes_content() {
        let report = sample_module_report();
        let global = sample_global_args();
        let args = sample_module_args(TableFormat::Md);
        let mut buf = Vec::new();

        write_module_report_to(&mut buf, &report, &global, &args).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(!output.is_empty(), "output must not be empty");
        assert!(output.contains("|Module|"), "must contain markdown header");
        assert!(output.contains("|crates/foo|"), "must contain module row");
        assert!(output.contains("|**Total**|"), "must contain total row");
    }

    #[test]
    fn write_module_report_to_tsv_writes_content() {
        let report = sample_module_report();
        let global = sample_global_args();
        let args = sample_module_args(TableFormat::Tsv);
        let mut buf = Vec::new();

        write_module_report_to(&mut buf, &report, &global, &args).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(!output.is_empty(), "output must not be empty");
        assert!(output.contains("Module\t"), "must contain TSV header");
        assert!(output.contains("crates/foo\t"), "must contain module row");
        assert!(output.contains("Total\t"), "must contain total row");
    }

    #[test]
    fn write_module_report_to_json_writes_receipt() {
        let report = sample_module_report();
        let global = sample_global_args();
        let args = sample_module_args(TableFormat::Json);
        let mut buf = Vec::new();

        write_module_report_to(&mut buf, &report, &global, &args).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(!output.is_empty(), "output must not be empty");
        // Parse as JSON to verify valid receipt
        let receipt: ModuleReceipt = serde_json::from_str(&output).unwrap();
        assert_eq!(receipt.mode, "module");
        assert_eq!(receipt.report.rows.len(), 2);
        assert_eq!(receipt.report.total.code, 1000);
    }
}

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
pub mod readme_doctests {}
