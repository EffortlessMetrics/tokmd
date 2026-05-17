//! JSON file-level export rendering.
//!
//! This module owns the JSON export receipt envelope and bare row-array output.
//! The parent export module keeps format dispatch and the public test-helper
//! facade stable.

use std::io::Write;

use anyhow::Result;

use tokmd_settings::ScanOptions;
use tokmd_types::{ExportArgs, ExportData, ExportReceipt, ScanStatus, ToolInfo};

use crate::{now_ms, scan_args};

use super::meta::{args_meta_from_export, redacted_export_data, redacted_rows_for_json};

pub(super) fn write_export_json<W: Write>(
    out: &mut W,
    export: &ExportData,
    global: &ScanOptions,
    args: &ExportArgs,
) -> Result<()> {
    if args.meta {
        let receipt = ExportReceipt {
            schema_version: tokmd_types::SCHEMA_VERSION,
            generated_at_ms: now_ms(),
            tool: ToolInfo::current(),
            mode: "export".to_string(),
            status: ScanStatus::Complete,
            warnings: vec![],
            scan: scan_args(&args.paths, global, Some(args.redact)),
            args: args_meta_from_export(export, args),
            data: redacted_export_data(export, args.redact),
        };
        writeln!(out, "{}", serde_json::to_string(&receipt)?)?;
    } else {
        writeln!(
            out,
            "{}",
            serde_json::to_string(&redacted_rows_for_json(export, args.redact))?
        )?;
    }
    Ok(())
}
