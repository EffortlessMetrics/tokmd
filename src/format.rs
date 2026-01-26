use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use serde::Serialize;

use crate::cli::{
    ChildIncludeMode, ConfigMode, ExportArgs, ExportFormat, GlobalArgs, LangArgs, ModuleArgs,
    RedactMode, TableFormat,
};
use crate::model::{ExportData, FileKind, FileRow, LangReport, ModuleReport, Totals};

/// Increment when JSON/JSONL output shapes change.
const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize)]
struct ToolInfo {
    name: &'static str,
    version: &'static str,
}

#[derive(Debug, Clone, Serialize)]
struct ScanArgs {
    paths: Vec<String>,
    excluded: Vec<String>,
    config: ConfigMode,
    hidden: bool,
    no_ignore: bool,
    no_ignore_parent: bool,
    no_ignore_dot: bool,
    no_ignore_vcs: bool,
    treat_doc_strings_as_comments: bool,
}

fn tool_info() -> ToolInfo {
    ToolInfo {
        name: "tokmd",
        version: env!("CARGO_PKG_VERSION"),
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
        paths: paths
            .iter()
            .map(|p| p.display().to_string())
            .collect(),
        excluded: global.excluded.clone(),
        config: global.config,
        hidden: global.hidden,
        no_ignore: global.no_ignore,
        no_ignore_parent: global.no_ignore_parent,
        no_ignore_dot: global.no_ignore_dot,
        no_ignore_vcs: global.no_ignore_vcs,
        treat_doc_strings_as_comments: global.treat_doc_strings_as_comments,
    };

    if let Some(mode) = redact {
        if mode == RedactMode::Paths || mode == RedactMode::All {
            args.paths = args.paths.iter().map(|p| redact_path(p)).collect();
        }
    }
    args
}

// -----------------------
// Language summary output
// -----------------------

#[derive(Debug, Clone, Serialize)]
struct LangReceipt {
    schema_version: u32,
    generated_at_ms: u128,
    tool: ToolInfo,
    mode: &'static str,
    scan: ScanArgs,
    args: LangArgsMeta,
    rows: Vec<crate::model::LangRow>,
    total: Totals,
}

#[derive(Debug, Clone, Serialize)]
struct LangArgsMeta {
    top: usize,
    with_files: bool,
    children: crate::cli::ChildrenMode,
}

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
                mode: "lang",
                scan: scan_args(&args.paths, global, None),
                args: LangArgsMeta {
                    top: report.top,
                    with_files: report.with_files,
                    children: report.children,
                },
                rows: report.rows.clone(),
                total: report.total.clone(),
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

#[derive(Debug, Clone, Serialize)]
struct ModuleReceipt {
    schema_version: u32,
    generated_at_ms: u128,
    tool: ToolInfo,
    mode: &'static str,
    scan: ScanArgs,
    args: ModuleArgsMeta,
    rows: Vec<crate::model::ModuleRow>,
    total: Totals,
}

#[derive(Debug, Clone, Serialize)]
struct ModuleArgsMeta {
    top: usize,
    module_roots: Vec<String>,
    module_depth: usize,
    children: ChildIncludeMode,
}

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
                mode: "module",
                scan: scan_args(&args.paths, global, None),
                args: ModuleArgsMeta {
                    top: report.top,
                    module_roots: report.module_roots.clone(),
                    module_depth: report.module_depth,
                    children: report.children,
                },
                rows: report.rows.clone(),
                total: report.total.clone(),
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
    mode: &'static str,
    scan: ScanArgs,
    args: ExportArgsMeta,
}

#[derive(Debug, Clone, Serialize)]
struct ExportArgsMeta {
    format: ExportFormat,
    module_roots: Vec<String>,
    module_depth: usize,
    children: ChildIncludeMode,
    min_code: usize,
    max_rows: usize,
    redact: RedactMode,
    strip_prefix: Option<String>,
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
            mode: "export",
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
        #[derive(Serialize)]
        struct ExportReceipt {
            schema_version: u32,
            generated_at_ms: u128,
            tool: ToolInfo,
            mode: &'static str,
            scan: ScanArgs,
            args: ExportArgsMeta,
            rows: Vec<FileRow>,
        }

        let receipt = ExportReceipt {
            schema_version: SCHEMA_VERSION,
            generated_at_ms: now_ms(),
            tool: tool_info(),
            mode: "export",
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
            rows: redact_rows(&export.rows, args.redact),
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
