//! Diff workflow facade.

use anyhow::Result;
use tokmd_types::DiffReceipt;

use crate::load_lang_report;
use crate::settings::DiffSettings;

/// Runs the diff workflow comparing two receipts or paths.
///
/// # Arguments
///
/// * `settings` - Diff settings (from, to references)
///
/// # Returns
///
/// A `DiffReceipt` showing changes between the two states.
///
/// # Example
///
/// ```rust
/// use tokmd_core::{diff_workflow, settings::DiffSettings};
///
/// let settings = DiffSettings {
///     from: ".".to_string(), // compare current dir to itself as a quick test
///     to: ".".to_string(),
///     ..Default::default()
/// };
///
/// let receipt = diff_workflow(&settings).expect("Diff failed");
/// assert!(receipt.totals.delta_code == 0); // delta is zero
/// ```
pub fn diff_workflow(settings: &DiffSettings) -> Result<DiffReceipt> {
    let from_report = load_lang_report(&settings.from)?;
    let to_report = load_lang_report(&settings.to)?;

    let rows = tokmd_format::compute_diff_rows(&from_report, &to_report);
    let totals = tokmd_format::compute_diff_totals(&rows);

    Ok(tokmd_format::create_diff_receipt(
        &settings.from,
        &settings.to,
        rows,
        totals,
    ))
}
