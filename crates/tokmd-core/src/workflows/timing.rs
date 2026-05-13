//! Phase timing helpers for core inventory workflows.
//!
//! These helpers are opt-in measurement surfaces. They use the same scan,
//! model, and receipt builders as the normal workflows and return the same
//! receipts plus phase timings, without changing default receipt schemas or CLI
//! output.

use std::path::Path;
use std::time::Instant;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokmd_types::{ExportReceipt, LangReceipt, ModuleReceipt};

use crate::settings::{ExportSettings, LangSettings, ModuleSettings, ScanSettings};
use crate::{build_export_receipt, build_lang_receipt, build_module_receipt};

use super::{scan_paths_or_current_dir, settings_to_scan_options};

/// Timing evidence for one core workflow run.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkflowTiming {
    /// Schema identifier for downstream tools.
    pub schema: String,
    /// Schema version for this timing record.
    pub schema_version: u32,
    /// `lang`, `module`, or `export`.
    pub workflow: String,
    /// Number of caller-provided scan roots. Paths are deliberately omitted.
    pub path_count: usize,
    /// Number of language entries returned by `tokei`.
    pub language_count: usize,
    /// Number of rows in the workflow's final report/data section.
    pub row_count: usize,
    /// Time spent in `tokmd-scan`.
    pub scan_ms: u128,
    /// Time spent in `tokmd-model` aggregation/selection.
    pub model_ms: u128,
    /// Time spent constructing the final receipt object.
    pub receipt_ms: u128,
    /// End-to-end time measured around the workflow body.
    pub total_ms: u128,
}

/// A normal workflow receipt plus opt-in timing evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimedWorkflow<T> {
    pub receipt: T,
    pub timing: WorkflowTiming,
}

/// Runs the language workflow and records scan/model/receipt phase timings.
///
/// # Errors
/// Returns any error produced by the underlying scan or model workflow.
pub fn timed_lang_workflow(
    scan: &ScanSettings,
    lang: &LangSettings,
) -> Result<TimedWorkflow<LangReceipt>> {
    let total_start = Instant::now();
    let scan_opts = settings_to_scan_options(scan);
    let paths = scan_paths_or_current_dir(scan);

    let scan_start = Instant::now();
    let languages = tokmd_scan::scan(&paths, &scan_opts)?;
    let scan_ms = elapsed_ms(scan_start);
    let language_count = languages.len();

    let model_start = Instant::now();
    let report = tokmd_model::create_lang_report(&languages, lang.top, lang.files, lang.children);
    let row_count = report.rows.len();
    let model_ms = elapsed_ms(model_start);

    let receipt_start = Instant::now();
    let receipt = build_lang_receipt(&paths, &scan_opts, lang, report);
    let receipt_ms = elapsed_ms(receipt_start);

    Ok(TimedWorkflow {
        receipt,
        timing: timing_record(TimingRecordInput {
            workflow: "lang",
            path_count: paths.len(),
            language_count,
            row_count,
            scan_ms,
            model_ms,
            receipt_ms,
            total_ms: elapsed_ms(total_start),
        }),
    })
}

/// Runs the module workflow and records scan/model/receipt phase timings.
///
/// # Errors
/// Returns any error produced by the underlying scan or model workflow.
pub fn timed_module_workflow(
    scan: &ScanSettings,
    module: &ModuleSettings,
) -> Result<TimedWorkflow<ModuleReceipt>> {
    let total_start = Instant::now();
    let scan_opts = settings_to_scan_options(scan);
    let paths = scan_paths_or_current_dir(scan);

    let scan_start = Instant::now();
    let languages = tokmd_scan::scan(&paths, &scan_opts)?;
    let scan_ms = elapsed_ms(scan_start);
    let language_count = languages.len();

    let model_start = Instant::now();
    let report = tokmd_model::create_module_report(
        &languages,
        &module.module_roots,
        module.module_depth,
        module.children,
        module.top,
    );
    let row_count = report.rows.len();
    let model_ms = elapsed_ms(model_start);

    let receipt_start = Instant::now();
    let receipt = build_module_receipt(&paths, &scan_opts, module, report);
    let receipt_ms = elapsed_ms(receipt_start);

    Ok(TimedWorkflow {
        receipt,
        timing: timing_record(TimingRecordInput {
            workflow: "module",
            path_count: paths.len(),
            language_count,
            row_count,
            scan_ms,
            model_ms,
            receipt_ms,
            total_ms: elapsed_ms(total_start),
        }),
    })
}

/// Runs the export workflow and records scan/model/receipt phase timings.
///
/// # Errors
/// Returns any error produced by the underlying scan or model workflow.
pub fn timed_export_workflow(
    scan: &ScanSettings,
    export: &ExportSettings,
) -> Result<TimedWorkflow<ExportReceipt>> {
    let total_start = Instant::now();
    let scan_opts = settings_to_scan_options(scan);
    let paths = scan_paths_or_current_dir(scan);
    let strip_prefix = export.strip_prefix.as_deref();

    let scan_start = Instant::now();
    let languages = tokmd_scan::scan(&paths, &scan_opts)?;
    let scan_ms = elapsed_ms(scan_start);
    let language_count = languages.len();

    let model_start = Instant::now();
    let data = tokmd_model::create_export_data(
        &languages,
        &export.module_roots,
        export.module_depth,
        export.children,
        strip_prefix.map(Path::new),
        export.min_code,
        export.max_rows,
    );
    let row_count = data.rows.len();
    let model_ms = elapsed_ms(model_start);

    let receipt_start = Instant::now();
    let receipt = build_export_receipt(&paths, &scan_opts, export, data);
    let receipt_ms = elapsed_ms(receipt_start);

    Ok(TimedWorkflow {
        receipt,
        timing: timing_record(TimingRecordInput {
            workflow: "export",
            path_count: paths.len(),
            language_count,
            row_count,
            scan_ms,
            model_ms,
            receipt_ms,
            total_ms: elapsed_ms(total_start),
        }),
    })
}

struct TimingRecordInput {
    workflow: &'static str,
    path_count: usize,
    language_count: usize,
    row_count: usize,
    scan_ms: u128,
    model_ms: u128,
    receipt_ms: u128,
    total_ms: u128,
}

fn timing_record(input: TimingRecordInput) -> WorkflowTiming {
    WorkflowTiming {
        schema: "tokmd.workflow_timing.v1".to_string(),
        schema_version: 1,
        workflow: input.workflow.to_string(),
        path_count: input.path_count,
        language_count: input.language_count,
        row_count: input.row_count,
        scan_ms: input.scan_ms,
        model_ms: input.model_ms,
        receipt_ms: input.receipt_ms,
        total_ms: input.total_ms,
    }
}

fn elapsed_ms(start: Instant) -> u128 {
    start.elapsed().as_millis()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use anyhow::Result;

    use super::*;

    #[test]
    fn timed_lang_workflow_preserves_receipt_and_records_phases() -> Result<()> {
        let temp = tempfile::tempdir()?;
        fs::write(temp.path().join("main.rs"), "fn main() {}\n")?;
        let scan = ScanSettings::for_paths(vec![temp.path().to_string_lossy().into_owned()]);
        let result = timed_lang_workflow(&scan, &LangSettings::default())?;

        assert_eq!(result.receipt.mode, "lang");
        assert_eq!(result.timing.workflow, "lang");
        assert_eq!(result.timing.path_count, 1);
        assert_eq!(result.timing.language_count, 1);
        assert_eq!(result.timing.row_count, result.receipt.report.rows.len());
        assert_eq!(result.timing.schema, "tokmd.workflow_timing.v1");
        Ok(())
    }

    #[test]
    fn timed_module_and_export_workflows_record_rows_without_paths() -> Result<()> {
        let temp = tempfile::tempdir()?;
        fs::create_dir_all(temp.path().join("src"))?;
        fs::write(temp.path().join("src").join("lib.rs"), "pub fn lib() {}\n")?;
        let scan = ScanSettings::for_paths(vec![temp.path().to_string_lossy().into_owned()]);

        let module = timed_module_workflow(&scan, &ModuleSettings::default())?;
        let export = timed_export_workflow(&scan, &ExportSettings::default())?;

        assert_eq!(module.timing.workflow, "module");
        assert_eq!(module.timing.row_count, module.receipt.report.rows.len());
        assert_eq!(export.timing.workflow, "export");
        assert_eq!(export.timing.row_count, export.receipt.data.rows.len());
        assert_eq!(export.timing.path_count, 1);
        Ok(())
    }
}
