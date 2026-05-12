//! Language and module summary JSON receipt rendering.
//!
//! This module owns JSON receipt construction for direct command output and
//! run-command artifact files. The parent summary module keeps public dispatch.

use std::fs::File;
use std::io::Write;
use std::path::Path;

use anyhow::Result;
use tokmd_settings::ScanOptions;
use tokmd_types::{
    LangArgs, LangArgsMeta, LangReceipt, LangReport, ModuleArgs, ModuleArgsMeta, ModuleReceipt,
    ModuleReport, RedactMode, ScanArgs, ScanStatus, ToolInfo,
};

use crate::{now_ms, redact_module_roots, scan_args, short_hash};

pub(super) fn write_lang_json<W: Write>(
    mut out: W,
    report: &LangReport,
    global: &ScanOptions,
    args: &LangArgs,
) -> Result<()> {
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
    Ok(())
}

pub(super) fn write_module_json<W: Write>(
    mut out: W,
    report: &ModuleReport,
    global: &ScanOptions,
    args: &ModuleArgs,
) -> Result<()> {
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
    Ok(())
}

pub(super) fn write_lang_json_to_file(
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

pub(super) fn write_module_json_to_file(
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
