//! In-memory analysis export preparation.

use std::path::PathBuf;

use anyhow::Result;
use tokmd_settings::ScanOptions;
use tokmd_types::{ChildIncludeMode, ExportData, ExportReceipt, FileRow};

use crate::settings::ExportSettings;
use crate::{InMemoryFile, build_export_receipt};

use super::super::{collect_pure_in_memory_rows, strip_virtual_export_prefix};

pub(super) struct PreparedAnalysisInput {
    pub(super) export_receipt: ExportReceipt,
    pub(super) logical_inputs: Vec<String>,
    pub(super) root: PathBuf,
    // Keeps the temporary materialized scan root alive while analysis reads it.
    pub(super) materialized_scan: Option<tokmd_scan::MaterializedScan>,
}

pub(super) fn prepare_rootless_in_memory_export(
    inputs: &[InMemoryFile],
    scan_opts: &ScanOptions,
    export: &ExportSettings,
) -> Result<PreparedAnalysisInput> {
    let (paths, rows) = collect_pure_in_memory_rows(
        inputs,
        scan_opts,
        &export.module_roots,
        export.module_depth,
        export.children,
    )?;
    let data = tokmd_model::create_export_data_from_rows(
        rows,
        &export.module_roots,
        export.module_depth,
        export.children,
        export.min_code,
        export.max_rows,
    );
    let logical_inputs: Vec<String> = paths
        .iter()
        .map(|path| tokmd_model::normalize_path(path, None))
        .collect();
    let export_receipt = build_export_receipt(&paths, scan_opts, export, data);

    Ok(PreparedAnalysisInput {
        export_receipt,
        logical_inputs,
        root: PathBuf::new(),
        materialized_scan: None,
    })
}

pub(super) fn prepare_materialized_in_memory_export(
    inputs: &[InMemoryFile],
    scan_opts: &ScanOptions,
    export: &ExportSettings,
) -> Result<PreparedAnalysisInput> {
    let scan = tokmd_scan::scan_in_memory(inputs, scan_opts)?;
    let data = collect_materialized_export_data(&scan, export);
    let logical_inputs: Vec<String> = scan
        .logical_paths()
        .iter()
        .map(|path| tokmd_model::normalize_path(path, None))
        .collect();
    let root = scan.strip_prefix().to_path_buf();
    let export_receipt = build_export_receipt(scan.logical_paths(), scan_opts, export, data);

    Ok(PreparedAnalysisInput {
        export_receipt,
        logical_inputs,
        root,
        materialized_scan: Some(scan),
    })
}

fn collect_materialized_rows(
    scan: &tokmd_scan::MaterializedScan,
    module_roots: &[String],
    module_depth: usize,
    children: ChildIncludeMode,
) -> Vec<FileRow> {
    tokmd_model::collect_file_rows(
        scan.languages(),
        module_roots,
        module_depth,
        children,
        Some(scan.strip_prefix()),
    )
}

fn collect_materialized_export_data(
    scan: &tokmd_scan::MaterializedScan,
    export: &ExportSettings,
) -> ExportData {
    let mut rows = collect_materialized_rows(
        scan,
        &export.module_roots,
        export.module_depth,
        export.children,
    );

    if let Some(strip_prefix) = export.strip_prefix.as_deref() {
        rows = strip_virtual_export_prefix(
            rows,
            strip_prefix,
            &export.module_roots,
            export.module_depth,
        );
    }

    tokmd_model::create_export_data_from_rows(
        rows,
        &export.module_roots,
        export.module_depth,
        export.children,
        export.min_code,
        export.max_rows,
    )
}
