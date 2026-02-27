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
//! * Redaction integration (via tokmd-redact re-exports)
//! * ScanArgs construction (single source of truth)
//!
//! ## What does NOT belong here
//! * Business logic (calculating stats)
//! * CLI argument parsing
//! * Analysis-specific formatting (use tokmd-analysis-format)

use std::borrow::Cow;
use std::fmt::Write as FmtWrite;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

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

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

/// Normalize a path to forward slashes and strip leading `./` for cross-platform stability.
///
/// This is the canonical normalization function for scan inputs. Use this
/// before storing paths in receipts to ensure consistent output across OS.
pub fn normalize_scan_input(p: &Path) -> String {
    let mut s = p.display().to_string().replace('\\', "/");
    while s.starts_with("./") {
        s = s.strip_prefix("./").unwrap().to_string();
    }
    if s.is_empty() { ".".to_string() } else { s }
}

/// Construct `ScanArgs` with optional redaction applied.
///
/// This is the single source of truth for building `ScanArgs` from CLI inputs.
/// All commands that produce receipts should use this function to ensure
/// consistent redaction and normalization behavior.
///
/// # Redaction Behavior
///
/// - `None` or `Some(RedactMode::None)`: Paths shown as-is (normalized only)
/// - `Some(RedactMode::Paths)`: Hash file paths, preserve extension
/// - `Some(RedactMode::All)`: Hash paths and excluded patterns
pub fn scan_args(paths: &[PathBuf], global: &ScanOptions, redact: Option<RedactMode>) -> ScanArgs {
    let should_redact = redact == Some(RedactMode::Paths) || redact == Some(RedactMode::All);
    let excluded_redacted = should_redact && !global.excluded.is_empty();

    let mut args = ScanArgs {
        paths: paths.iter().map(|p| normalize_scan_input(p)).collect(),
        excluded: if should_redact {
            global.excluded.iter().map(|p| short_hash(p)).collect()
        } else {
            global.excluded.clone()
        },
        excluded_redacted,
        config: global.config,
        hidden: global.hidden,
        no_ignore: global.no_ignore,
        no_ignore_parent: global.no_ignore || global.no_ignore_parent,
        no_ignore_dot: global.no_ignore || global.no_ignore_dot,
        no_ignore_vcs: global.no_ignore || global.no_ignore_vcs,
        treat_doc_strings_as_comments: global.treat_doc_strings_as_comments,
    };

    if should_redact {
        args.paths = args.paths.iter().map(|p| redact_path(p)).collect();
    }
    args
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
    let mut s = String::new();

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
    let mut s = String::new();

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
    let mut s = String::new();
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
    let mut s = String::new();
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
                module_roots: export.module_roots.clone(),
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
                module_roots: export.module_roots.clone(),
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
            serde_json::to_string(&redact_rows(&export.rows, args.redact).collect::<Vec<_>>())?
        )?;
    }
    Ok(())
}

fn redact_rows(rows: &[FileRow], mode: RedactMode) -> impl Iterator<Item = Cow<'_, FileRow>> {
    rows.iter().map(move |r| {
        if mode == RedactMode::None {
            Cow::Borrowed(r)
        } else {
            let mut owned = r.clone();
            if mode == RedactMode::Paths || mode == RedactMode::All {
                owned.path = redact_path(&owned.path);
            }
            if mode == RedactMode::All {
                owned.module = short_hash(&owned.module);
            }
            Cow::Owned(owned)
        }
    })
}

// Re-export redaction functions for backwards compatibility
pub use tokmd_redact::{redact_path, short_hash};

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
    let timestamp = OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string());

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
        serial_number: format!("urn:uuid:{}", uuid::Uuid::new_v4()),
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
) -> Result<()> {
    let receipt = ModuleReceipt {
        schema_version: tokmd_types::SCHEMA_VERSION,
        generated_at_ms: now_ms(),
        tool: ToolInfo::current(),
        mode: "module".to_string(),
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

    let meta = ExportMeta {
        ty: "meta",
        schema_version: tokmd_types::SCHEMA_VERSION,
        generated_at_ms: now_ms(),
        tool: ToolInfo::current(),
        mode: "export".to_string(),
        status: ScanStatus::Complete,
        warnings: vec![],
        scan: scan.clone(),
        args: args_meta.clone(),
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

// -----------------
// Diff output
// -----------------

use tokmd_types::{DiffReceipt, DiffRow, DiffTotals};

/// Compute diff rows from two lang reports.
pub fn compute_diff_rows(from_report: &LangReport, to_report: &LangReport) -> Vec<DiffRow> {
    use std::collections::{BTreeMap, BTreeSet};

    let mut all_langs = BTreeSet::new();
    let mut old_map = BTreeMap::new();
    for row in &from_report.rows {
        all_langs.insert(row.lang.as_str());
        old_map.entry(row.lang.as_str()).or_insert(row);
    }
    let mut new_map = BTreeMap::new();
    for row in &to_report.rows {
        all_langs.insert(row.lang.as_str());
        new_map.entry(row.lang.as_str()).or_insert(row);
    }

    all_langs
        .into_iter()
        .filter_map(|lang_name| {
            let old = old_map.get(lang_name);
            let new = new_map.get(lang_name);

            let (old_code, old_lines, old_files, old_bytes, old_tokens) = match old {
                Some(r) => (r.code, r.lines, r.files, r.bytes, r.tokens),
                None => (0, 0, 0, 0, 0),
            };
            let (new_code, new_lines, new_files, new_bytes, new_tokens) = match new {
                Some(r) => (r.code, r.lines, r.files, r.bytes, r.tokens),
                None => (0, 0, 0, 0, 0),
            };

            // Skip if no change
            if old_code == new_code
                && old_lines == new_lines
                && old_files == new_files
                && old_bytes == new_bytes
                && old_tokens == new_tokens
            {
                return None;
            }

            Some(DiffRow {
                lang: lang_name.to_string(),
                old_code,
                new_code,
                delta_code: new_code as i64 - old_code as i64,
                old_lines,
                new_lines,
                delta_lines: new_lines as i64 - old_lines as i64,
                old_files,
                new_files,
                delta_files: new_files as i64 - old_files as i64,
                old_bytes,
                new_bytes,
                delta_bytes: new_bytes as i64 - old_bytes as i64,
                old_tokens,
                new_tokens,
                delta_tokens: new_tokens as i64 - old_tokens as i64,
            })
        })
        .collect()
}

/// Compute totals from diff rows.
pub fn compute_diff_totals(rows: &[DiffRow]) -> DiffTotals {
    let mut totals = DiffTotals {
        old_code: 0,
        new_code: 0,
        delta_code: 0,
        old_lines: 0,
        new_lines: 0,
        delta_lines: 0,
        old_files: 0,
        new_files: 0,
        delta_files: 0,
        old_bytes: 0,
        new_bytes: 0,
        delta_bytes: 0,
        old_tokens: 0,
        new_tokens: 0,
        delta_tokens: 0,
    };

    for row in rows {
        totals.old_code += row.old_code;
        totals.new_code += row.new_code;
        totals.delta_code += row.delta_code;
        totals.old_lines += row.old_lines;
        totals.new_lines += row.new_lines;
        totals.delta_lines += row.delta_lines;
        totals.old_files += row.old_files;
        totals.new_files += row.new_files;
        totals.delta_files += row.delta_files;
        totals.old_bytes += row.old_bytes;
        totals.new_bytes += row.new_bytes;
        totals.delta_bytes += row.delta_bytes;
        totals.old_tokens += row.old_tokens;
        totals.new_tokens += row.new_tokens;
        totals.delta_tokens += row.delta_tokens;
    }

    totals
}

fn format_delta(delta: i64) -> String {
    if delta > 0 {
        format!("+{}", delta)
    } else {
        delta.to_string()
    }
}

/// Render diff as Markdown table.
pub fn render_diff_md(
    from_source: &str,
    to_source: &str,
    rows: &[DiffRow],
    totals: &DiffTotals,
) -> String {
    let mut s = String::new();

    let _ = writeln!(s, "## Diff: {} → {}", from_source, to_source);
    s.push('\n');

    // Summary comparison table
    s.push_str("### Summary\n\n");
    s.push_str("|Metric|From|To|Delta|Change|\n");
    s.push_str("|---|---:|---:|---:|---:|\n");

    let old_total = totals.old_code as f64;
    let new_total = totals.new_code as f64;
    let delta = totals.delta_code;
    let change_pct = if old_total > 0.0 {
        ((new_total - old_total) / old_total) * 100.0
    } else if new_total > 0.0 {
        100.0
    } else {
        0.0
    };

    let _ = writeln!(
        s,
        "|Total LOC|{}|{}|{}|{:+.1}%|",
        totals.old_code,
        totals.new_code,
        format_delta(delta),
        change_pct
    );
    s.push('\n');

    // Detailed language breakdown
    s.push_str("### Language Breakdown\n\n");
    s.push_str("|Language|Old LOC|New LOC|Delta|\n");
    s.push_str("|---|---:|---:|---:|\n");

    for row in rows {
        let _ = writeln!(
            s,
            "|{}|{}|{}|{}|",
            row.lang,
            row.old_code,
            row.new_code,
            format_delta(row.delta_code)
        );
    }

    let _ = writeln!(
        s,
        "|**Total**|{}|{}|{}|",
        totals.old_code,
        totals.new_code,
        format_delta(totals.delta_code)
    );

    s
}

/// Create a DiffReceipt for JSON output.
pub fn create_diff_receipt(
    from_source: &str,
    to_source: &str,
    rows: Vec<DiffRow>,
    totals: DiffTotals,
) -> DiffReceipt {
    DiffReceipt {
        schema_version: tokmd_types::SCHEMA_VERSION,
        generated_at_ms: now_ms(),
        tool: ToolInfo::current(),
        mode: "diff".to_string(),
        from_source: from_source.to_string(),
        to_source: to_source.to_string(),
        diff_rows: rows,
        totals,
    }
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
        fn redact_rows_paths_end_with_extension(ext in "[a-z]{1,4}") {
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

    #[test]
    fn test_render_diff_md_smoke() {
        // Kills mutants: render_diff_md -> String::new() / "xyzzy".into()
        let from = LangReport {
            rows: vec![LangRow {
                lang: "Rust".to_string(),
                code: 10,
                lines: 10,
                files: 1,
                bytes: 100,
                tokens: 20,
                avg_lines: 10,
            }],
            total: Totals {
                code: 10,
                lines: 10,
                files: 1,
                bytes: 100,
                tokens: 20,
                avg_lines: 10,
            },
            with_files: false,
            children: ChildrenMode::Collapse,
            top: 0,
        };

        let to = LangReport {
            rows: vec![LangRow {
                lang: "Rust".to_string(),
                code: 12,
                lines: 12,
                files: 1,
                bytes: 120,
                tokens: 24,
                avg_lines: 12,
            }],
            total: Totals {
                code: 12,
                lines: 12,
                files: 1,
                bytes: 120,
                tokens: 24,
                avg_lines: 12,
            },
            with_files: false,
            children: ChildrenMode::Collapse,
            top: 0,
        };

        let rows = compute_diff_rows(&from, &to);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].lang, "Rust");
        assert_eq!(rows[0].delta_code, 2);

        let totals = compute_diff_totals(&rows);
        assert_eq!(totals.delta_code, 2);

        let md = render_diff_md("from", "to", &rows, &totals);

        assert!(!md.trim().is_empty(), "diff markdown must not be empty");
        assert!(md.contains("from"));
        assert!(md.contains("to"));
        assert!(md.contains("Rust"));
    }

    #[test]
    fn test_compute_diff_rows_language_added() {
        // Tests language being added (was 0, now has code)
        let from = LangReport {
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

        let to = LangReport {
            rows: vec![LangRow {
                lang: "Python".to_string(),
                code: 100,
                lines: 120,
                files: 5,
                bytes: 5000,
                tokens: 250,
                avg_lines: 24,
            }],
            total: Totals {
                code: 100,
                lines: 120,
                files: 5,
                bytes: 5000,
                tokens: 250,
                avg_lines: 24,
            },
            with_files: false,
            children: ChildrenMode::Collapse,
            top: 0,
        };

        let rows = compute_diff_rows(&from, &to);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].lang, "Python");
        assert_eq!(rows[0].old_code, 0);
        assert_eq!(rows[0].new_code, 100);
        assert_eq!(rows[0].delta_code, 100);
    }

    #[test]
    fn test_compute_diff_rows_language_removed() {
        // Tests language being removed (had code, now 0)
        let from = LangReport {
            rows: vec![LangRow {
                lang: "Go".to_string(),
                code: 50,
                lines: 60,
                files: 2,
                bytes: 2000,
                tokens: 125,
                avg_lines: 30,
            }],
            total: Totals {
                code: 50,
                lines: 60,
                files: 2,
                bytes: 2000,
                tokens: 125,
                avg_lines: 30,
            },
            with_files: false,
            children: ChildrenMode::Collapse,
            top: 0,
        };

        let to = LangReport {
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

        let rows = compute_diff_rows(&from, &to);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].lang, "Go");
        assert_eq!(rows[0].old_code, 50);
        assert_eq!(rows[0].new_code, 0);
        assert_eq!(rows[0].delta_code, -50);
    }

    #[test]
    fn test_compute_diff_rows_unchanged_excluded() {
        // Tests that unchanged languages are excluded from diff
        let report = LangReport {
            rows: vec![LangRow {
                lang: "Rust".to_string(),
                code: 100,
                lines: 100,
                files: 1,
                bytes: 1000,
                tokens: 250,
                avg_lines: 100,
            }],
            total: Totals {
                code: 100,
                lines: 100,
                files: 1,
                bytes: 1000,
                tokens: 250,
                avg_lines: 100,
            },
            with_files: false,
            children: ChildrenMode::Collapse,
            top: 0,
        };

        let rows = compute_diff_rows(&report, &report);
        assert!(rows.is_empty(), "unchanged languages should be excluded");
    }

    #[test]
    fn test_format_delta() {
        // Kills mutants in format_delta function
        assert_eq!(format_delta(5), "+5");
        assert_eq!(format_delta(0), "0");
        assert_eq!(format_delta(-3), "-3");
    }

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
