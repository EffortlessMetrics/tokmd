//! JSONL file-level export rendering.
//!
//! This module owns the JSONL meta envelope, row wrapper, and file writer used
//! by the `run` command. The parent export module keeps format dispatch and the
//! public test-helper facade stable.

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use anyhow::Result;
use serde::Serialize;

use tokmd_settings::ScanOptions;
use tokmd_types::{
    ExportArgs, ExportArgsMeta, ExportData, FileRow, RedactMode, ScanArgs, ScanStatus, ToolInfo,
};

use crate::{now_ms, redact_module_roots, redact_path, scan_args};

use super::redact_rows;

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

pub(super) fn write_export_jsonl<W: Write>(
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

    write_rows(out, export, args.redact)
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

    write_rows(&mut out, export, args_meta.redact)?;
    out.flush()?;
    Ok(())
}

fn write_rows<W: Write>(out: &mut W, export: &ExportData, redact: RedactMode) -> Result<()> {
    for row in redact_rows(&export.rows, redact) {
        let wrapper = JsonlRow {
            ty: "row",
            row: &row,
        };
        writeln!(out, "{}", serde_json::to_string(&wrapper)?)?;
    }
    Ok(())
}
