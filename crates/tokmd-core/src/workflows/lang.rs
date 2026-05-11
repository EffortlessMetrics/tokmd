//! Language summary workflow facade.

use anyhow::Result;
use tokmd_settings::ScanOptions;
use tokmd_types::{ChildIncludeMode, LangReceipt};

use crate::settings::{LangSettings, ScanSettings};
use crate::{InMemoryFile, build_lang_receipt};

use super::{
    collect_pure_in_memory_rows, deterministic_in_memory_scan_options, scan_paths_or_current_dir,
    settings_to_scan_options,
};

/// Runs the language summary workflow with pure settings types.
///
/// This is the binding-friendly API that doesn't require Clap types.
///
/// # Arguments
///
/// * `scan` - Scan settings (paths, exclusions, etc.)
/// * `lang` - Language-specific settings (top N, files, etc.)
///
/// # Returns
///
/// A `LangReceipt` containing the language summary.
///
/// # Example
///
/// ```rust
/// use std::fs;
///
/// use tokmd_core::{
///     lang_workflow,
///     settings::{LangSettings, ScanSettings},
/// };
///
/// let tmp = tempfile::tempdir().expect("tempdir");
/// fs::write(tmp.path().join("main.rs"), "fn main() {}").expect("write fixture");
///
/// let scan = ScanSettings::for_paths(vec![tmp.path().to_string_lossy().into_owned()]);
/// let lang = LangSettings::default();
///
/// let receipt = lang_workflow(&scan, &lang).expect("language scan");
/// assert_eq!(receipt.report.rows.len(), 1);
/// ```
pub fn lang_workflow(scan: &ScanSettings, lang: &LangSettings) -> Result<LangReceipt> {
    let scan_opts = settings_to_scan_options(scan);
    let paths = scan_paths_or_current_dir(scan);

    let languages = tokmd_scan::scan(&paths, &scan_opts)?;
    let report = tokmd_model::create_lang_report(&languages, lang.top, lang.files, lang.children);

    Ok(build_lang_receipt(&paths, &scan_opts, lang, report))
}

/// Runs the language summary workflow for ordered in-memory inputs.
///
/// # Example
///
/// ```rust
/// use tokmd_core::{
///     InMemoryFile, lang_workflow_from_inputs,
///     settings::{LangSettings, ScanOptions},
/// };
///
/// let inputs = vec![InMemoryFile::new("src/main.rs", b"fn main() {}".to_vec())];
/// let scan_opts = ScanOptions::default();
/// let lang = LangSettings::default();
///
/// let receipt = lang_workflow_from_inputs(&inputs, &scan_opts, &lang).expect("Scan failed");
/// assert_eq!(receipt.report.rows.len(), 1);
/// ```
pub fn lang_workflow_from_inputs(
    inputs: &[InMemoryFile],
    scan_opts: &ScanOptions,
    lang: &LangSettings,
) -> Result<LangReceipt> {
    let scan_opts = deterministic_in_memory_scan_options(scan_opts);
    let (paths, rows) =
        collect_pure_in_memory_rows(inputs, &scan_opts, &[], 1, ChildIncludeMode::Separate)?;
    let report =
        tokmd_model::create_lang_report_from_rows(&rows, lang.top, lang.files, lang.children);

    Ok(build_lang_receipt(&paths, &scan_opts, lang, report))
}
