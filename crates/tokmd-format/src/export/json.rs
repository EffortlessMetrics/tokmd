//! JSON file-level export rendering.
//!
//! This module owns the JSON export receipt envelope and bare row-array output.
//! The parent export module keeps format dispatch and the public test-helper
//! facade stable.

use std::io::Write;

use anyhow::Result;

use tokmd_settings::ScanOptions;
use tokmd_types::{
    ExportArgs, ExportArgsMeta, ExportData, ExportReceipt, RedactMode, ScanStatus, ToolInfo,
};

use crate::{now_ms, redact_module_roots, redact_path, scan_args};

use super::redact_rows;

pub(super) fn write_export_json<W: Write>(
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
