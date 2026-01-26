//! # tokmd-format
//!
//! **Tier 3 (Formatting)**
//!
//! This crate handles the rendering and serialization of `tokmd` receipts.
//! It supports Markdown, TSV, JSON, JSONL, and CSV formats.
//!
//! ## What belongs here
//! * Serialization logic (JSON/CSV)
//! * Markdown template rendering
//! * Output file writing
//! * Redaction hashing logic (for output safety)
//!
//! ## What does NOT belong here
//! * Business logic (calculating stats)
//! * CLI arg parsing

use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use serde::Serialize;

use tokmd_config::{ExportFormat, GlobalArgs, RedactMode, TableFormat};
use tokmd_types::{
    ExportArgs, ExportArgsMeta, ExportData, ExportReceipt, FileKind, FileRow, LangArgs,
    LangArgsMeta, LangReceipt, LangReport, ModuleArgs, ModuleArgsMeta, ModuleReceipt, ModuleReport,
    ScanArgs, ScanStatus, ToolInfo,
};

/// Increment when JSON/JSONL output shapes change.
const SCHEMA_VERSION: u32 = 2;

fn tool_info() -> ToolInfo {
    ToolInfo {
        name: "tokmd".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn scan_args(paths: &[PathBuf], global: &GlobalArgs, redact: Option<RedactMode>) -> ScanArgs {
    let mut args = ScanArgs {
        paths: paths.iter().map(|p| p.display().to_string()).collect(),
        excluded: global.excluded.clone(),
        config: global.config,
        hidden: global.hidden,
        no_ignore: global.no_ignore,
        no_ignore_parent: global.no_ignore_parent,
        no_ignore_dot: global.no_ignore_dot,
        no_ignore_vcs: global.no_ignore_vcs,
        treat_doc_strings_as_comments: global.treat_doc_strings_as_comments,
    };

    if redact == Some(RedactMode::Paths) || redact == Some(RedactMode::All) {
        args.paths = args.paths.iter().map(|p| redact_path(p)).collect();
    }
    args
}

// -----------------------
// Language summary output
// -----------------------

pub fn print_lang_report(report: &LangReport, global: &GlobalArgs, args: &LangArgs) -> Result<()> {
    match args.format {
        TableFormat::Md => {
            print!("{}", render_lang_md(report));
        }
        TableFormat::Tsv => {
            print!("{}", render_lang_tsv(report));
        }
        TableFormat::Json => {
            let receipt = LangReceipt {
                schema_version: SCHEMA_VERSION,
                generated_at_ms: now_ms(),
                tool: tool_info(),
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
            println!("{}", serde_json::to_string(&receipt)?);
        }
    }
    Ok(())
}

fn render_lang_md(report: &LangReport) -> String {
    let mut s = String::new();

    if report.with_files {
        s.push_str("|Lang|Code|Lines|Files|Avg|\n");
        s.push_str("|---|---:|---:|---:|---:|\n");
        for r in &report.rows {
            s.push_str(&format!(
                "|{}|{}|{}|{}|{}|\n",
                r.lang, r.code, r.lines, r.files, r.avg_lines
            ));
        }
        s.push_str(&format!(
            "|**Total**|{}|{}|{}|{}|\n",
            report.total.code, report.total.lines, report.total.files, report.total.avg_lines
        ));
    } else {
        s.push_str("|Lang|Code|Lines|\n");
        s.push_str("|---|---:|---:|\n");
        for r in &report.rows {
            s.push_str(&format!("|{}|{}|{}|\n", r.lang, r.code, r.lines));
        }
        s.push_str(&format!(
            "|**Total**|{}|{}|\n",
            report.total.code, report.total.lines
        ));
    }

    s
}

fn render_lang_tsv(report: &LangReport) -> String {
    let mut s = String::new();

    if report.with_files {
        s.push_str("Lang\tCode\tLines\tFiles\tAvg\n");
        for r in &report.rows {
            s.push_str(&format!(
                "{}\t{}\t{}\t{}\t{}\n",
                r.lang, r.code, r.lines, r.files, r.avg_lines
            ));
        }
        s.push_str(&format!(
            "Total\t{}\t{}\t{}\t{}\n",
            report.total.code, report.total.lines, report.total.files, report.total.avg_lines
        ));
    } else {
        s.push_str("Lang\tCode\tLines\n");
        for r in &report.rows {
            s.push_str(&format!("{}\t{}\t{}\n", r.lang, r.code, r.lines));
        }
        s.push_str(&format!(
            "Total\t{}\t{}\n",
            report.total.code, report.total.lines
        ));
    }

    s
}

// ---------------------
// Module summary output
// ---------------------

pub fn print_module_report(
    report: &ModuleReport,
    global: &GlobalArgs,
    args: &ModuleArgs,
) -> Result<()> {
    match args.format {
        TableFormat::Md => {
            print!("{}", render_module_md(report));
        }
        TableFormat::Tsv => {
            print!("{}", render_module_tsv(report));
        }
        TableFormat::Json => {
            let receipt = ModuleReceipt {
                schema_version: SCHEMA_VERSION,
                generated_at_ms: now_ms(),
                tool: tool_info(),
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
            println!("{}", serde_json::to_string(&receipt)?);
        }
    }
    Ok(())
}

fn render_module_md(report: &ModuleReport) -> String {
    let mut s = String::new();
    s.push_str("|Module|Code|Lines|Files|Avg|\n");
    s.push_str("|---|---:|---:|---:|---:|\n");
    for r in &report.rows {
        s.push_str(&format!(
            "|{}|{}|{}|{}|{}|\n",
            r.module, r.code, r.lines, r.files, r.avg_lines
        ));
    }
    s.push_str(&format!(
        "|**Total**|{}|{}|{}|{}|\n",
        report.total.code, report.total.lines, report.total.files, report.total.avg_lines
    ));
    s
}

fn render_module_tsv(report: &ModuleReport) -> String {
    let mut s = String::new();
    s.push_str("Module\tCode\tLines\tFiles\tAvg\n");
    for r in &report.rows {
        s.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\n",
            r.module, r.code, r.lines, r.files, r.avg_lines
        ));
    }
    s.push_str(&format!(
        "Total\t{}\t{}\t{}\t{}\n",
        report.total.code, report.total.lines, report.total.files, report.total.avg_lines
    ));
    s
}

// -----------------
// Export (datasets)
// -----------------

#[derive(Debug, Clone, Serialize)]
struct ExportMeta {
    #[serde(rename = "type")]
    ty: &'static str,
    schema_version: u32,
    generated_at_ms: u128,
    tool: ToolInfo,
    mode: String,
    status: ScanStatus,
    warnings: Vec<String>,
    scan: ScanArgs,
    args: ExportArgsMeta,
}

#[derive(Debug, Clone, Serialize)]
struct JsonlRow<'a> {
    #[serde(rename = "type")]
    ty: &'static str,
    #[serde(flatten)]
    row: &'a FileRow,
}

pub fn write_export(export: &ExportData, global: &GlobalArgs, args: &ExportArgs) -> Result<()> {
    match &args.out {
        Some(path) => {
            let file = File::create(path)?;
            let mut out = BufWriter::new(file);
            write_export_to(&mut out, export, global, args)?;
            out.flush()?;
        }
        None => {
            let stdout = io::stdout();
            let mut out = stdout.lock();
            write_export_to(&mut out, export, global, args)?;
            out.flush()?;
        }
    }
    Ok(())
}

fn write_export_to<W: Write>(
    out: &mut W,
    export: &ExportData,
    global: &GlobalArgs,
    args: &ExportArgs,
) -> Result<()> {
    match args.format {
        ExportFormat::Csv => write_export_csv(out, export, args),
        ExportFormat::Jsonl => write_export_jsonl(out, export, global, args),
        ExportFormat::Json => write_export_json(out, export, global, args),
    }
}

fn write_export_csv<W: Write>(out: &mut W, export: &ExportData, args: &ExportArgs) -> Result<()> {
    let mut wtr = csv::WriterBuilder::new().has_headers(true).from_writer(out);
    wtr.write_record([
        "path", "module", "lang", "kind", "code", "comments", "blanks", "lines",
    ])?;

    for r in redact_rows(&export.rows, args.redact) {
        wtr.write_record([
            r.path,
            r.module,
            r.lang,
            match r.kind {
                FileKind::Parent => "parent".to_string(),
                FileKind::Child => "child".to_string(),
            },
            r.code.to_string(),
            r.comments.to_string(),
            r.blanks.to_string(),
            r.lines.to_string(),
        ])?;
    }

    wtr.flush()?;
    Ok(())
}

fn write_export_jsonl<W: Write>(
    out: &mut W,
    export: &ExportData,
    global: &GlobalArgs,
    args: &ExportArgs,
) -> Result<()> {
    if args.meta {
        let meta = ExportMeta {
            ty: "meta",
            schema_version: SCHEMA_VERSION,
            generated_at_ms: now_ms(),
            tool: tool_info(),
            mode: "export".to_string(),
            status: ScanStatus::Complete,
            warnings: vec![],
            scan: scan_args(&args.paths, global, Some(args.redact)),
            args: ExportArgsMeta {
                format: args.format,
                module_roots: export.module_roots.clone(),
                module_depth: export.module_depth,
                children: export.children,
                min_code: args.min_code,
                max_rows: args.max_rows,
                redact: args.redact,
                strip_prefix: args.strip_prefix.as_ref().map(|p| p.display().to_string()),
            },
        };
        writeln!(out, "{}", serde_json::to_string(&meta)?)?;
    }

    for row in redact_rows(&export.rows, args.redact) {
        let wrapper = JsonlRow {
            ty: "row",
            row: &row,
        };
        writeln!(out, "{}", serde_json::to_string(&wrapper)?)?;
    }
    Ok(())
}

fn write_export_json<W: Write>(
    out: &mut W,
    export: &ExportData,
    global: &GlobalArgs,
    args: &ExportArgs,
) -> Result<()> {
    if args.meta {
        let receipt = ExportReceipt {
            schema_version: SCHEMA_VERSION,
            generated_at_ms: now_ms(),
            tool: tool_info(),
            mode: "export".to_string(),
            status: ScanStatus::Complete,
            warnings: vec![],
            scan: scan_args(&args.paths, global, Some(args.redact)),
            args: ExportArgsMeta {
                format: args.format,
                module_roots: export.module_roots.clone(),
                module_depth: export.module_depth,
                children: export.children,
                min_code: args.min_code,
                max_rows: args.max_rows,
                redact: args.redact,
                strip_prefix: args.strip_prefix.as_ref().map(|p| p.display().to_string()),
            },
            data: ExportData {
                rows: redact_rows(&export.rows, args.redact),
                module_roots: export.module_roots.clone(),
                module_depth: export.module_depth,
                children: export.children,
            },
        };
        writeln!(out, "{}", serde_json::to_string(&receipt)?)?;
    } else {
        writeln!(
            out,
            "{}",
            serde_json::to_string(&redact_rows(&export.rows, args.redact))?
        )?;
    }
    Ok(())
}

fn redact_rows(rows: &[FileRow], mode: RedactMode) -> Vec<FileRow> {
    if mode == RedactMode::None {
        return rows.to_vec();
    }

    rows.iter()
        .cloned()
        .map(|mut r| {
            if mode == RedactMode::Paths || mode == RedactMode::All {
                r.path = redact_path(&r.path);
            }
            if mode == RedactMode::All {
                r.module = short_hash(&r.module);
            }
            r
        })
        .collect()
}

fn short_hash(s: &str) -> String {
    let mut hex = blake3::hash(s.as_bytes()).to_hex().to_string();
    hex.truncate(16);
    hex
}

fn redact_path(path: &str) -> String {
    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    let mut out = short_hash(path);
    if !ext.is_empty() {
        out.push('.');
        out.push_str(ext);
    }
    out
}
