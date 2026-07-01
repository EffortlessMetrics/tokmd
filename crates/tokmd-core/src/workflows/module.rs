//! Module summary workflow facade.

use anyhow::Result;
use tokmd_settings::ScanOptions;
use tokmd_types::ModuleReceipt;

use crate::settings::{ModuleSettings, ScanSettings};
use crate::{InMemoryFile, build_module_receipt};

use super::{
    collect_pure_in_memory_rows, deterministic_in_memory_scan_options, scan_paths_or_current_dir,
    settings_to_scan_options, single_scan_root_strip_prefix,
};

/// Runs the module summary workflow with pure settings types.
///
/// # Arguments
///
/// * `scan` - Scan settings (paths, exclusions, etc.)
/// * `module` - Module-specific settings (roots, depth, etc.)
///
/// # Returns
///
/// A `ModuleReceipt` containing the module breakdown.
///
/// # Example
///
/// ```rust
/// use tokmd_core::{module_workflow, settings::{ScanSettings, ModuleSettings}};
///
/// let scan = ScanSettings::current_dir();
/// let module = ModuleSettings {
///     module_depth: 2,
///     ..Default::default()
/// };
///
/// let receipt = module_workflow(&scan, &module).expect("Module scan failed");
/// assert!(receipt.report.rows.len() > 0);
/// ```
pub fn module_workflow(scan: &ScanSettings, module: &ModuleSettings) -> Result<ModuleReceipt> {
    let scan_opts = settings_to_scan_options(scan);
    let paths = scan_paths_or_current_dir(scan);
    let strip_prefix = single_scan_root_strip_prefix(&paths);

    let languages = tokmd_scan::scan(&paths, &scan_opts)?;
    let file_rows = tokmd_model::collect_file_rows(
        &languages,
        &module.module_roots,
        module.module_depth,
        module.children,
        strip_prefix,
    );
    let report = tokmd_model::create_module_report_from_rows(
        &file_rows,
        &module.module_roots,
        module.module_depth,
        module.children,
        module.top,
    );

    Ok(build_module_receipt(&paths, &scan_opts, module, report))
}

/// Runs the module summary workflow for ordered in-memory inputs.
///
/// # Example
///
/// ```rust
/// use tokmd_core::{
///     InMemoryFile, module_workflow_from_inputs,
///     settings::{ModuleSettings, ScanOptions},
/// };
///
/// let inputs = vec![InMemoryFile::new("src/main.rs", b"fn main() {}".to_vec())];
/// let scan_opts = ScanOptions::default();
/// let module = ModuleSettings::default();
///
/// let receipt =
///     module_workflow_from_inputs(&inputs, &scan_opts, &module).expect("Module scan failed");
/// assert_eq!(receipt.report.rows.len(), 1);
/// ```
pub fn module_workflow_from_inputs(
    inputs: &[InMemoryFile],
    scan_opts: &ScanOptions,
    module: &ModuleSettings,
) -> Result<ModuleReceipt> {
    let scan_opts = deterministic_in_memory_scan_options(scan_opts);
    let (paths, rows) = collect_pure_in_memory_rows(
        inputs,
        &scan_opts,
        &module.module_roots,
        module.module_depth,
        module.children,
    )?;
    let report = tokmd_model::create_module_report_from_rows(
        &rows,
        &module.module_roots,
        module.module_depth,
        module.children,
        module.top,
    );

    Ok(build_module_receipt(&paths, &scan_opts, module, report))
}
