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

use std::borrow::Cow;
use std::fmt::Write as FmtWrite;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(test)]
use std::path::PathBuf;

use anyhow::Result;
use serde::Serialize;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use tokmd_settings::ScanOptions;
use tokmd_types::{
    ExportArgs, ExportArgsMeta, ExportData, ExportFormat, ExportReceipt, FileKind, FileRow,
    LangArgs, LangArgsMeta, LangReceipt, LangReport, ModuleArgs, ModuleArgsMeta, ModuleReceipt,
    ModuleReport, RedactMode, ScanArgs, ScanStatus, TableFormat, ToolInfo,
};

pub mod analysis;
pub mod badge;
mod diff;
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

pub fn write_export(export: &ExportData, global: &ScanOptions, args: &ExportArgs) -> Result<()> {
    match &args.output {
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
    global: &ScanOptions,
    args: &ExportArgs,
) -> Result<()> {
    match args.format {
        ExportFormat::Csv => write_export_csv(out, export, args),
        ExportFormat::Jsonl => write_export_jsonl(out, export, global, args),
        ExportFormat::Json => write_export_json(out, export, global, args),
        ExportFormat::Cyclonedx => write_export_cyclonedx(out, export, args.redact),
    }
}

fn write_export_csv<W: Write>(out: &mut W, export: &ExportData, args: &ExportArgs) -> Result<()> {
    let mut wtr = csv::WriterBuilder::new().has_headers(true).from_writer(out);
    wtr.write_record([
        "path", "module", "lang", "kind", "code", "comments", "blanks", "lines", "bytes", "tokens",
    ])?;

    for r in redact_rows(&export.rows, args.redact) {
        let code = r.code.to_string();
        let comments = r.comments.to_string();
        let blanks = r.blanks.to_string();
        let lines = r.lines.to_string();
        let bytes = r.bytes.to_string();
        let tokens = r.tokens.to_string();
        let kind = match r.kind {
            FileKind::Parent => "parent",
            FileKind::Child => "child",
        };

        wtr.write_record([
            r.path.as_str(),
            r.module.as_str(),
            r.lang.as_str(),
            kind,
            &code,
            &comments,
            &blanks,
            &lines,
            &bytes,
            &tokens,
        ])?;
    }

    wtr.flush()?;
    Ok(())
}

fn write_export_jsonl<W: Write>(
    out: &mut W,
    export: &ExportData,
    global: &ScanOptions,
    args: &ExportArgs,
) -> Result<()> {
    let module_roots = redact_module_roots(&export.module_roots, args.redact);

    if args.meta {
        let should_redact = args.redact == RedactMode::Paths || args.redact == RedactMode::All;
        let strip_prefix_redacted = should_redact && args.strip_prefix.is_some();

        let meta = ExportMeta {
            ty: "meta",
            schema_version: tokmd_types::SCHEMA_VERSION,
            generated_at_ms: now_ms(),
            tool: ToolInfo::current(),
            mode: "export".to_string(),
            status: ScanStatus::Complete,
            warnings: vec![],
            scan: scan_args(&args.paths, global, Some(args.redact)),
            args: ExportArgsMeta {
                format: args.format,
                module_roots: module_roots.clone(),
                module_depth: export.module_depth,
                children: export.children,
                min_code: args.min_code,
                max_rows: args.max_rows,
                redact: args.redact,
                strip_prefix: if should_redact {
                    args.strip_prefix
                        .as_ref()
                        .map(|p| redact_path(&p.display().to_string().replace('\\', "/")))
                } else {
                    args.strip_prefix
                        .as_ref()
                        .map(|p| p.display().to_string().replace('\\', "/"))
                },
                strip_prefix_redacted,
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
    global: &ScanOptions,
    args: &ExportArgs,
) -> Result<()> {
    let module_roots = redact_module_roots(&export.module_roots, args.redact);

    if args.meta {
        let should_redact = args.redact == RedactMode::Paths || args.redact == RedactMode::All;
        let strip_prefix_redacted = should_redact && args.strip_prefix.is_some();

        let receipt = ExportReceipt {
            schema_version: tokmd_types::SCHEMA_VERSION,
            generated_at_ms: now_ms(),
            tool: ToolInfo::current(),
            mode: "export".to_string(),
            status: ScanStatus::Complete,
            warnings: vec![],
            scan: scan_args(&args.paths, global, Some(args.redact)),
            args: ExportArgsMeta {
                format: args.format,
                module_roots: module_roots.clone(),
                module_depth: export.module_depth,
                children: export.children,
                min_code: args.min_code,
                max_rows: args.max_rows,
                redact: args.redact,
                strip_prefix: if should_redact {
                    args.strip_prefix
                        .as_ref()
                        .map(|p| redact_path(&p.display().to_string().replace('\\', "/")))
                } else {
                    args.strip_prefix
                        .as_ref()
                        .map(|p| p.display().to_string().replace('\\', "/"))
                },
                strip_prefix_redacted,
            },
            data: ExportData {
                rows: redact_rows(&export.rows, args.redact)
                    .map(|c| c.into_owned())
                    .collect(),
                module_roots: module_roots.clone(),
                module_depth: export.module_depth,
                children: export.children,
            },
        };
        writeln!(out, "{}", serde_json::to_string(&receipt)?)?;
    } else {
        writeln!(
            out,
            "{}",
            serde_json::to_string(&redact_rows(&export.rows, args.redact).collect::<Vec<_>>())?
        )?;
    }
    Ok(())
}

fn redact_rows(rows: &[FileRow], mode: RedactMode) -> impl Iterator<Item = Cow<'_, FileRow>> {
    rows.iter().map(move |r| match mode {
        RedactMode::None => Cow::Borrowed(r),
        RedactMode::Paths => Cow::Owned(FileRow {
            path: redact_path(&r.path),
            module: r.module.clone(),
            lang: r.lang.clone(),
            kind: r.kind,
            code: r.code,
            comments: r.comments,
            blanks: r.blanks,
            lines: r.lines,
            bytes: r.bytes,
            tokens: r.tokens,
        }),
        RedactMode::All => Cow::Owned(FileRow {
            path: redact_path(&r.path),
            module: short_hash(&r.module),
            lang: r.lang.clone(),
            kind: r.kind,
            code: r.code,
            comments: r.comments,
            blanks: r.blanks,
            lines: r.lines,
            bytes: r.bytes,
            tokens: r.tokens,
        }),
    })
}

// -----------------
// CycloneDX SBOM
// -----------------

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct CycloneDxBom {
    bom_format: &'static str,
    spec_version: &'static str,
    serial_number: String,
    version: u32,
    metadata: CycloneDxMetadata,
    components: Vec<CycloneDxComponent>,
}

#[derive(Debug, Clone, Serialize)]
struct CycloneDxMetadata {
    timestamp: String,
    tools: Vec<CycloneDxTool>,
}

#[derive(Debug, Clone, Serialize)]
struct CycloneDxTool {
    vendor: &'static str,
    name: &'static str,
    version: String,
}

#[derive(Debug, Clone, Serialize)]
struct CycloneDxComponent {
    #[serde(rename = "type")]
    ty: &'static str,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    group: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    properties: Vec<CycloneDxProperty>,
}

#[derive(Debug, Clone, Serialize)]
struct CycloneDxProperty {
    name: String,
    value: String,
}

fn write_export_cyclonedx<W: Write>(
    out: &mut W,
    export: &ExportData,
    redact: RedactMode,
) -> Result<()> {
    write_export_cyclonedx_impl(out, export, redact, None, None)
}

fn write_export_cyclonedx_impl<W: Write>(
    out: &mut W,
    export: &ExportData,
    redact: RedactMode,
    serial_number: Option<String>,
    timestamp: Option<String>,
) -> Result<()> {
    let timestamp = timestamp.unwrap_or_else(|| {
        OffsetDateTime::now_utc()
            .format(&Rfc3339)
            .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
    });

    // Apply redaction to rows before generating components
    let components: Vec<CycloneDxComponent> = redact_rows(&export.rows, redact)
        .map(|row| {
            let mut properties = vec![
                CycloneDxProperty {
                    name: "tokmd:lang".to_string(),
                    value: row.lang.clone(),
                },
                CycloneDxProperty {
                    name: "tokmd:code".to_string(),
                    value: row.code.to_string(),
                },
                CycloneDxProperty {
                    name: "tokmd:comments".to_string(),
                    value: row.comments.to_string(),
                },
                CycloneDxProperty {
                    name: "tokmd:blanks".to_string(),
                    value: row.blanks.to_string(),
                },
                CycloneDxProperty {
                    name: "tokmd:lines".to_string(),
                    value: row.lines.to_string(),
                },
                CycloneDxProperty {
                    name: "tokmd:bytes".to_string(),
                    value: row.bytes.to_string(),
                },
                CycloneDxProperty {
                    name: "tokmd:tokens".to_string(),
                    value: row.tokens.to_string(),
                },
            ];

            // Add kind if it's a child
            if row.kind == FileKind::Child {
                properties.push(CycloneDxProperty {
                    name: "tokmd:kind".to_string(),
                    value: "child".to_string(),
                });
            }

            CycloneDxComponent {
                ty: "file",
                name: row.path.clone(),
                group: if row.module.is_empty() {
                    None
                } else {
                    Some(row.module.clone())
                },
                properties,
            }
        })
        .collect();

    let bom = CycloneDxBom {
        bom_format: "CycloneDX",
        spec_version: "1.6",
        serial_number: serial_number
            .unwrap_or_else(|| format!("urn:uuid:{}", uuid::Uuid::new_v4())),
        version: 1,
        metadata: CycloneDxMetadata {
            timestamp,
            tools: vec![CycloneDxTool {
                vendor: "tokmd",
                name: "tokmd",
                version: env!("CARGO_PKG_VERSION").to_string(),
            }],
        },
        components,
    };

    writeln!(out, "{}", serde_json::to_string_pretty(&bom)?)?;
    Ok(())
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

/// Write export data as JSONL to a file path.
///
/// This is a convenience function for the `run` command that accepts
/// pre-constructed `ScanArgs` and `ExportArgsMeta` rather than requiring
/// the full `ScanOptions` and `ExportArgs` structs.
pub fn write_export_jsonl_to_file(
    path: &Path,
    export: &ExportData,
    scan: &ScanArgs,
    args_meta: &ExportArgsMeta,
) -> Result<()> {
    let file = File::create(path)?;
    let mut out = BufWriter::new(file);

    let mut final_args = args_meta.clone();
    final_args.module_roots = redact_module_roots(&final_args.module_roots, args_meta.redact);

    let meta = ExportMeta {
        ty: "meta",
        schema_version: tokmd_types::SCHEMA_VERSION,
        generated_at_ms: now_ms(),
        tool: ToolInfo::current(),
        mode: "export".to_string(),
        status: ScanStatus::Complete,
        warnings: vec![],
        scan: scan.clone(),
        args: final_args,
    };
    writeln!(out, "{}", serde_json::to_string(&meta)?)?;

    for row in redact_rows(&export.rows, args_meta.redact) {
        let wrapper = JsonlRow {
            ty: "row",
            row: &row,
        };
        writeln!(out, "{}", serde_json::to_string(&wrapper)?)?;
    }

    out.flush()?;
    Ok(())
}

// =============================================================================
// Public test helpers - expose internal functions for integration tests
// =============================================================================

/// Write CSV export to a writer (exposed for testing).
#[doc(hidden)]
pub fn write_export_csv_to<W: Write>(
    out: &mut W,
    export: &ExportData,
    args: &ExportArgs,
) -> Result<()> {
    write_export_csv(out, export, args)
}

/// Write JSONL export to a writer (exposed for testing).
#[doc(hidden)]
pub fn write_export_jsonl_to<W: Write>(
    out: &mut W,
    export: &ExportData,
    global: &ScanOptions,
    args: &ExportArgs,
) -> Result<()> {
    write_export_jsonl(out, export, global, args)
}

/// Write JSON export to a writer (exposed for testing).
#[doc(hidden)]
pub fn write_export_json_to<W: Write>(
    out: &mut W,
    export: &ExportData,
    global: &ScanOptions,
    args: &ExportArgs,
) -> Result<()> {
    write_export_json(out, export, global, args)
}

/// Write CycloneDX export to a writer (exposed for testing).
#[doc(hidden)]
pub fn write_export_cyclonedx_to<W: Write>(
    out: &mut W,
    export: &ExportData,
    redact: RedactMode,
) -> Result<()> {
    write_export_cyclonedx(out, export, redact)
}

/// Write CycloneDX export to a writer with explicit options (exposed for testing).
#[doc(hidden)]
pub fn write_export_cyclonedx_with_options<W: Write>(
    out: &mut W,
    export: &ExportData,
    redact: RedactMode,
    serial_number: Option<String>,
    timestamp: Option<String>,
) -> Result<()> {
    write_export_cyclonedx_impl(out, export, redact, serial_number, timestamp)
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

    fn sample_file_rows() -> Vec<FileRow> {
        vec![
            FileRow {
                path: "src/lib.rs".to_string(),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code: 100,
                comments: 20,
                blanks: 10,
                lines: 130,
                bytes: 1000,
                tokens: 250,
            },
            FileRow {
                path: "tests/test.rs".to_string(),
                module: "tests".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code: 50,
                comments: 5,
                blanks: 5,
                lines: 60,
                bytes: 500,
                tokens: 125,
            },
        ]
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
    // Redaction Tests
    // ========================

    #[test]
    fn redact_rows_none_mode() {
        let rows = sample_file_rows();
        let redacted: Vec<_> = redact_rows(&rows, RedactMode::None).collect();

        // Should be identical
        assert_eq!(redacted.len(), rows.len());
        assert_eq!(redacted[0].path, "src/lib.rs");
        assert_eq!(redacted[0].module, "src");
    }

    #[test]
    fn redact_rows_paths_mode() {
        let rows = sample_file_rows();
        let redacted: Vec<_> = redact_rows(&rows, RedactMode::Paths).collect();

        // Paths should be redacted (16 char hash + extension)
        assert_ne!(redacted[0].path, "src/lib.rs");
        assert!(redacted[0].path.ends_with(".rs"));
        assert_eq!(redacted[0].path.len(), 16 + 3); // hash + ".rs"

        // Module should NOT be redacted
        assert_eq!(redacted[0].module, "src");
    }

    #[test]
    fn redact_rows_all_mode() {
        let rows = sample_file_rows();
        let redacted: Vec<_> = redact_rows(&rows, RedactMode::All).collect();

        // Paths should be redacted
        assert_ne!(redacted[0].path, "src/lib.rs");
        assert!(redacted[0].path.ends_with(".rs"));

        // Module should ALSO be redacted (16 char hash)
        assert_ne!(redacted[0].module, "src");
        assert_eq!(redacted[0].module.len(), 16);
    }

    #[test]
    fn redact_rows_preserves_other_fields() {
        let rows = sample_file_rows();
        let redacted: Vec<_> = redact_rows(&rows, RedactMode::All).collect();

        // All other fields should be preserved
        assert_eq!(redacted[0].lang, "Rust");
        assert_eq!(redacted[0].kind, FileKind::Parent);
        assert_eq!(redacted[0].code, 100);
        assert_eq!(redacted[0].comments, 20);
        assert_eq!(redacted[0].blanks, 10);
        assert_eq!(redacted[0].lines, 130);
        assert_eq!(redacted[0].bytes, 1000);
        assert_eq!(redacted[0].tokens, 250);
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

        #[test]
        fn redact_rows_preserves_count(
            code in 0usize..10000,
            comments in 0usize..1000,
            blanks in 0usize..500
        ) {
            let rows = vec![FileRow {
                path: "test/file.rs".to_string(),
                module: "test".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code,
                comments,
                blanks,
                lines: code + comments + blanks,
                bytes: 1000,
                tokens: 250,
            }];

            for mode in [RedactMode::None, RedactMode::Paths, RedactMode::All] {
                let redacted: Vec<_> = redact_rows(&rows, mode).collect();
                prop_assert_eq!(redacted.len(), 1);
                prop_assert_eq!(redacted[0].code, code);
                prop_assert_eq!(redacted[0].comments, comments);
                prop_assert_eq!(redacted[0].blanks, blanks);
            }
        }

        #[test]
        fn redact_rows_paths_preserve_allowlisted_extensions(ext in "rs|js|ts|json|md|toml|gz") {
            let path = format!("some/path/file.{}", ext);
            let rows = vec![FileRow {
                path: path.clone(),
                module: "some".to_string(),
                lang: "Test".to_string(),
                kind: FileKind::Parent,
                code: 100,
                comments: 10,
                blanks: 5,
                lines: 115,
                bytes: 1000,
                tokens: 250,
            }];

            let redacted: Vec<_> = redact_rows(&rows, RedactMode::Paths).collect();
            prop_assert!(redacted[0].path.ends_with(&format!(".{}", ext)),
                "Redacted path '{}' should end with .{}", redacted[0].path, ext);
        }

        #[test]
        fn redact_rows_paths_strip_untrusted_extensions(ext in "passwd|secret|pass1234|token") {
            let path = format!("some/path/file.{}", ext);
            let rows = vec![FileRow {
                path: path.clone(),
                module: "some".to_string(),
                lang: "Test".to_string(),
                kind: FileKind::Parent,
                code: 100,
                comments: 10,
                blanks: 5,
                lines: 115,
                bytes: 1000,
                tokens: 250,
            }];

            let redacted: Vec<_> = redact_rows(&rows, RedactMode::Paths).collect();
            prop_assert_eq!(redacted[0].path.len(), 16);
            prop_assert!(!redacted[0].path.contains('.'));
            prop_assert!(!redacted[0].path.contains(&ext));
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
