use std::io::{self, Write};
use std::path::Path;

use anyhow::Result;

use tokmd_settings::ScanOptions;
use tokmd_types::{
    LangArgs, LangArgsMeta, LangReport, ModuleArgs, ModuleArgsMeta, ModuleReport, RedactMode,
    ScanArgs, TableFormat,
};

mod json;
mod lang;
mod module;

use json::{write_lang_json, write_module_json};
use lang::{render_lang_md, render_lang_tsv};
use module::{render_module_md, render_module_tsv};

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
            write_lang_json(out, report, global, args)?;
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
            write_module_json(out, report, global, args)?;
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
    json::write_lang_json_to_file(path, report, scan, args_meta)
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
    json::write_module_json_to_file(path, report, scan, args_meta, redact)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tokmd_settings::ChildrenMode;
    use tokmd_types::{LangReceipt, LangRow, ModuleReceipt, ModuleRow, Totals};

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

    #[test]
    fn render_lang_md_without_files() {
        let report = sample_lang_report(false);
        let output = render_lang_md(&report);

        assert!(output.contains("|Lang|Code|Lines|Bytes|Tokens|"));
        assert!(!output.contains("|Files|"));
        assert!(!output.contains("|Avg|"));
        assert!(output.contains("|Rust|1000|1200|50000|2500|"));
        assert!(output.contains("|TOML|50|60|1000|125|"));
        assert!(output.contains("|**Total**|1050|1260|51000|2625|"));
    }

    #[test]
    fn render_lang_md_with_files() {
        let report = sample_lang_report(true);
        let output = render_lang_md(&report);

        assert!(output.contains("|Lang|Code|Lines|Files|Bytes|Tokens|Avg|"));
        assert!(output.contains("|Rust|1000|1200|10|50000|2500|120|"));
        assert!(output.contains("|TOML|50|60|2|1000|125|30|"));
        assert!(output.contains("|**Total**|1050|1260|12|51000|2625|105|"));
    }

    #[test]
    fn render_lang_md_table_structure() {
        let report = sample_lang_report(true);
        let output = render_lang_md(&report);

        let lines: Vec<&str> = output.lines().collect();
        assert!(lines.len() >= 4);
        assert!(lines[1].contains("|---|"));
        assert!(lines[1].contains(':'));
    }

    #[test]
    fn render_lang_tsv_without_files() {
        let report = sample_lang_report(false);
        let output = render_lang_tsv(&report);

        assert!(output.starts_with("Lang\tCode\tLines\tBytes\tTokens\n"));
        assert!(!output.contains("\tFiles\t"));
        assert!(!output.contains("\tAvg"));
        assert!(output.contains("Rust\t1000\t1200\t50000\t2500"));
        assert!(output.contains("TOML\t50\t60\t1000\t125"));
        assert!(output.contains("Total\t1050\t1260\t51000\t2625"));
    }

    #[test]
    fn render_lang_tsv_with_files() {
        let report = sample_lang_report(true);
        let output = render_lang_tsv(&report);

        assert!(output.starts_with("Lang\tCode\tLines\tFiles\tBytes\tTokens\tAvg\n"));
        assert!(output.contains("Rust\t1000\t1200\t10\t50000\t2500\t120"));
        assert!(output.contains("TOML\t50\t60\t2\t1000\t125\t30"));
    }

    #[test]
    fn render_lang_tsv_tab_separated() {
        let report = sample_lang_report(false);
        let output = render_lang_tsv(&report);

        for line in output.lines().skip(1) {
            if line.starts_with("Total") || line.starts_with("Rust") || line.starts_with("TOML") {
                assert_eq!(line.matches('\t').count(), 4);
            }
        }
    }

    #[test]
    fn render_module_md_structure() {
        let report = sample_module_report();
        let output = render_module_md(&report);

        assert!(output.contains("|Module|Code|Lines|Files|Bytes|Tokens|Avg|"));
        assert!(output.contains("|crates/foo|800|950|8|40000|2000|119|"));
        assert!(output.contains("|crates/bar|200|250|2|10000|500|125|"));
        assert!(output.contains("|**Total**|1000|1200|10|50000|2500|120|"));
    }

    #[test]
    fn render_module_md_table_format() {
        let report = sample_module_report();
        let output = render_module_md(&report);

        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 5);
        assert!(lines[1].contains("---:"));
    }

    #[test]
    fn render_module_tsv_structure() {
        let report = sample_module_report();
        let output = render_module_tsv(&report);

        assert!(output.starts_with("Module\tCode\tLines\tFiles\tBytes\tTokens\tAvg\n"));
        assert!(output.contains("crates/foo\t800\t950\t8\t40000\t2000\t119"));
        assert!(output.contains("crates/bar\t200\t250\t2\t10000\t500\t125"));
        assert!(output.contains("Total\t1000\t1200\t10\t50000\t2500\t120"));
    }

    #[test]
    fn render_module_tsv_tab_count() {
        let report = sample_module_report();
        let output = render_module_tsv(&report);

        for line in output.lines() {
            assert_eq!(line.matches('\t').count(), 6);
        }
    }

    #[test]
    fn snapshot_lang_md_with_files() {
        let report = sample_lang_report(true);
        let output = render_lang_md(&report);
        insta::with_settings!({ prepend_module_to_snapshot => false }, {
            insta::assert_snapshot!("tokmd_format__tests__snapshot_lang_md_with_files", output);
        });
    }

    #[test]
    fn snapshot_lang_md_without_files() {
        let report = sample_lang_report(false);
        let output = render_lang_md(&report);
        insta::with_settings!({ prepend_module_to_snapshot => false }, {
            insta::assert_snapshot!("tokmd_format__tests__snapshot_lang_md_without_files", output);
        });
    }

    #[test]
    fn snapshot_lang_tsv_with_files() {
        let report = sample_lang_report(true);
        let output = render_lang_tsv(&report);
        insta::with_settings!({ prepend_module_to_snapshot => false }, {
            insta::assert_snapshot!("tokmd_format__tests__snapshot_lang_tsv_with_files", output);
        });
    }

    #[test]
    fn snapshot_module_md() {
        let report = sample_module_report();
        let output = render_module_md(&report);
        insta::with_settings!({ prepend_module_to_snapshot => false }, {
            insta::assert_snapshot!("tokmd_format__tests__snapshot_module_md", output);
        });
    }

    #[test]
    fn snapshot_module_tsv() {
        let report = sample_module_report();
        let output = render_module_tsv(&report);
        insta::with_settings!({ prepend_module_to_snapshot => false }, {
            insta::assert_snapshot!("tokmd_format__tests__snapshot_module_tsv", output);
        });
    }

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

        let receipt: ModuleReceipt = serde_json::from_str(&output).unwrap();
        assert_eq!(receipt.mode, "module");
        assert_eq!(receipt.report.rows.len(), 2);
        assert_eq!(receipt.report.total.code, 1000);
    }
}
