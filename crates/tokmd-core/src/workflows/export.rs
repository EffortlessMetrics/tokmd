//! File export workflow facade.

use std::path::Path;

use anyhow::Result;
use tokmd_settings::ScanOptions;
use tokmd_types::ExportReceipt;

use crate::settings::{ExportSettings, ScanSettings};
use crate::{InMemoryFile, build_export_receipt};

use super::{
    collect_pure_in_memory_rows, deterministic_in_memory_scan_options, scan_paths_or_current_dir,
    settings_to_scan_options, strip_virtual_export_prefix,
};

/// Runs the export workflow with pure settings types.
///
/// # Arguments
///
/// * `scan` - Scan settings (paths, exclusions, etc.)
/// * `export` - Export-specific settings (format, min_code, etc.)
///
/// # Returns
///
/// An `ExportReceipt` containing file-level data.
///
/// # Example
///
/// ```rust
/// use tokmd_core::{export_workflow, settings::{ScanSettings, ExportSettings}};
///
/// let scan = ScanSettings::current_dir();
/// let export = ExportSettings::default();
///
/// let receipt = export_workflow(&scan, &export).expect("Export scan failed");
/// assert!(receipt.data.rows.len() > 0);
/// ```
pub fn export_workflow(scan: &ScanSettings, export: &ExportSettings) -> Result<ExportReceipt> {
    let scan_opts = settings_to_scan_options(scan);
    let paths = scan_paths_or_current_dir(scan);
    let strip_prefix = export.strip_prefix.as_deref();

    let languages = tokmd_scan::scan(&paths, &scan_opts)?;
    let data = tokmd_model::create_export_data(
        &languages,
        &export.module_roots,
        export.module_depth,
        export.children,
        strip_prefix.map(Path::new),
        export.min_code,
        export.max_rows,
    );

    Ok(build_export_receipt(&paths, &scan_opts, export, data))
}

/// Runs the file export workflow for ordered in-memory inputs.
///
/// # Example
///
/// ```rust
/// use tokmd_core::{
///     InMemoryFile, export_workflow_from_inputs,
///     settings::{ExportSettings, ScanOptions},
/// };
///
/// let inputs = vec![InMemoryFile::new("src/main.rs", b"fn main() {}".to_vec())];
/// let scan_opts = ScanOptions::default();
/// let export = ExportSettings::default();
///
/// let receipt =
///     export_workflow_from_inputs(&inputs, &scan_opts, &export).expect("Export scan failed");
/// assert_eq!(receipt.data.rows.len(), 1);
/// ```
pub fn export_workflow_from_inputs(
    inputs: &[InMemoryFile],
    scan_opts: &ScanOptions,
    export: &ExportSettings,
) -> Result<ExportReceipt> {
    let scan_opts = deterministic_in_memory_scan_options(scan_opts);
    let (paths, mut rows) = collect_pure_in_memory_rows(
        inputs,
        &scan_opts,
        &export.module_roots,
        export.module_depth,
        export.children,
    )?;
    if let Some(strip_prefix) = export.strip_prefix.as_deref() {
        rows = strip_virtual_export_prefix(
            rows,
            strip_prefix,
            &export.module_roots,
            export.module_depth,
        );
    }
    let data = tokmd_model::create_export_data_from_rows(
        rows,
        &export.module_roots,
        export.module_depth,
        export.children,
        export.min_code,
        export.max_rows,
    );

    Ok(build_export_receipt(&paths, &scan_opts, export, data))
}
