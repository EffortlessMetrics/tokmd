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

use crate::{now_ms, scan_args};

use super::{
    meta::{args_meta_from_export, redacted_args_meta},
    redact_rows,
};

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

impl ExportMeta {
    fn complete(scan: ScanArgs, args: ExportArgsMeta) -> Self {
        Self {
            ty: "meta",
            schema_version: tokmd_types::SCHEMA_VERSION,
            generated_at_ms: now_ms(),
            tool: ToolInfo::current(),
            mode: "export".to_string(),
            status: ScanStatus::Complete,
            warnings: vec![],
            scan,
            args,
        }
    }
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
    if args.meta {
        let meta = ExportMeta::complete(
            scan_args(&args.paths, global, Some(args.redact)),
            args_meta_from_export(export, args),
        );
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

    let meta = ExportMeta::complete(scan.clone(), redacted_args_meta(args_meta));
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
