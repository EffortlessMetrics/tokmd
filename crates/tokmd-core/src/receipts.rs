//! Receipt construction helpers for core workflows.

use std::path::PathBuf;
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
use std::time::{SystemTime, UNIX_EPOCH};

use tokmd_format::scan_args;
use tokmd_settings::ScanOptions;
use tokmd_types::{
    ExportArgsMeta, ExportData, ExportReceipt, LangArgsMeta, LangReceipt, LangReport,
    ModuleArgsMeta, ModuleReceipt, ModuleReport, RedactMode, SCHEMA_VERSION, ScanStatus, ToolInfo,
};

use crate::settings::{ExportSettings, LangSettings, ModuleSettings};

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
#[inline]
fn now_ms() -> u128 {
    // Keep wasm receipts from reusing zero as a fake wall-clock sentinel.
    js_sys::Date::now().max(1.0) as u128
}

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
#[inline]
fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

pub(crate) fn build_lang_receipt(
    paths: &[PathBuf],
    scan_opts: &ScanOptions,
    lang: &LangSettings,
    report: LangReport,
) -> LangReceipt {
    LangReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: now_ms(),
        tool: ToolInfo::current(),
        mode: "lang".to_string(),
        status: ScanStatus::Complete,
        warnings: vec![],
        scan: scan_args(paths, scan_opts, lang.redact),
        args: LangArgsMeta {
            format: "json".to_string(),
            top: lang.top,
            with_files: lang.files,
            children: lang.children,
        },
        report,
    }
}

pub(crate) fn build_module_receipt(
    paths: &[PathBuf],
    scan_opts: &ScanOptions,
    module: &ModuleSettings,
    report: ModuleReport,
) -> ModuleReceipt {
    ModuleReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: now_ms(),
        tool: ToolInfo::current(),
        mode: "module".to_string(),
        status: ScanStatus::Complete,
        warnings: vec![],
        scan: scan_args(paths, scan_opts, module.redact),
        args: ModuleArgsMeta {
            format: "json".to_string(),
            top: module.top,
            module_roots: module.module_roots.clone(),
            module_depth: module.module_depth,
            children: module.children,
        },
        report,
    }
}

pub(crate) fn build_export_receipt(
    paths: &[PathBuf],
    scan_opts: &ScanOptions,
    export: &ExportSettings,
    data: ExportData,
) -> ExportReceipt {
    let should_redact = export.redact == RedactMode::Paths || export.redact == RedactMode::All;
    let strip_prefix_redacted = should_redact && export.strip_prefix.is_some();

    ExportReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: now_ms(),
        tool: ToolInfo::current(),
        mode: "export".to_string(),
        status: ScanStatus::Complete,
        warnings: vec![],
        scan: scan_args(paths, scan_opts, Some(export.redact)),
        args: ExportArgsMeta {
            format: export.format,
            module_roots: export.module_roots.clone(),
            module_depth: export.module_depth,
            children: export.children,
            min_code: export.min_code,
            max_rows: export.max_rows,
            redact: export.redact,
            strip_prefix: if should_redact {
                export
                    .strip_prefix
                    .as_ref()
                    .map(|p| tokmd_format::redact_path(p))
            } else {
                export.strip_prefix.clone()
            },
            strip_prefix_redacted,
        },
        data: redact_export_data(data, export.redact),
    }
}

/// Apply redaction to export data.
fn redact_export_data(data: ExportData, mode: RedactMode) -> ExportData {
    if mode == RedactMode::None {
        return data;
    }

    let rows = data
        .rows
        .into_iter()
        .map(|mut row| {
            if mode == RedactMode::Paths || mode == RedactMode::All {
                row.path = tokmd_format::redact_path(&row.path);
            }
            if mode == RedactMode::All {
                row.module = tokmd_format::short_hash(&row.module);
            }
            row
        })
        .collect();

    ExportData {
        rows,
        module_roots: data.module_roots,
        module_depth: data.module_depth,
        children: data.children,
    }
}
