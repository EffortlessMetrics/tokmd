//! Analysis workflow facade.

use std::path::PathBuf;

use anyhow::Result;
use tokmd_analysis as analysis;
use tokmd_analysis_types::{AnalysisReceipt, AnalysisSource};
use tokmd_settings::ScanOptions;
use tokmd_types::{ChildIncludeMode, ExportData, ExportReceipt, FileRow};

use crate::settings::{AnalyzeSettings, ExportSettings, ScanSettings};
use crate::{InMemoryFile, build_export_receipt};

use super::{
    collect_pure_in_memory_rows, deterministic_in_memory_scan_options, strip_virtual_export_prefix,
};

use super::export_workflow;

mod request;

use request::build_analysis_request;
#[cfg(test)]
pub(crate) use request::{parse_analysis_preset, parse_effort_request};

/// Analyze workflow (requires `analysis` feature).
///
/// Runs export + analysis workflows and returns an `AnalysisReceipt`.
///
/// # Example
///
/// ```rust
/// use tokmd_core::{analyze_workflow, settings::{ScanSettings, AnalyzeSettings}};
///
/// let scan = ScanSettings::current_dir();
/// let analyze = AnalyzeSettings {
///     preset: "receipt".to_string(),
///     ..Default::default()
/// };
///
/// let receipt = analyze_workflow(&scan, &analyze).expect("Analyze scan failed");
/// assert!(receipt.derived.is_some());
/// ```
pub fn analyze_workflow(scan: &ScanSettings, analyze: &AnalyzeSettings) -> Result<AnalysisReceipt> {
    let export_receipt = export_workflow(scan, &ExportSettings::default())?;
    let root = derive_analysis_root(scan)
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."));

    analyze_with_export_receipt(export_receipt, scan.paths.clone(), root, analyze)
}

/// Analyze workflow for ordered in-memory inputs (requires `analysis` feature).
///
/// Runs the in-memory export + analysis pipeline and returns an `AnalysisReceipt`.
///
/// `preset = "receipt"` and `preset = "estimate"` stay on the pure row path
/// and do not borrow the host repository as a fake root. Richer presets still
/// materialize a temporary scan root until the remaining analysis seams are
/// moved off the filesystem.
///
/// # Example
///
/// ```rust
/// use tokmd_core::{analyze_workflow_from_inputs, settings::{AnalyzeSettings, ScanOptions}, InMemoryFile};
///
/// let inputs = vec![
///     InMemoryFile {
///         path: "src/main.rs".into(),
///         bytes: b"fn main() { println!(\"hello world\"); }".to_vec(),
///     }
/// ];
///
/// let scan_opts = ScanOptions::default();
/// let analyze_opts = AnalyzeSettings {
///     preset: "receipt".to_string(),
///     ..Default::default()
/// };
///
/// let receipt = analyze_workflow_from_inputs(&inputs, &scan_opts, &analyze_opts)
///     .expect("analyze_workflow_from_inputs failed");
/// assert!(receipt.derived.is_some());
/// ```
pub fn analyze_workflow_from_inputs(
    inputs: &[InMemoryFile],
    scan_opts: &ScanOptions,
    analyze: &AnalyzeSettings,
) -> Result<AnalysisReceipt> {
    let export = ExportSettings::default();
    let scan_opts = deterministic_in_memory_scan_options(scan_opts);
    if supports_rootless_in_memory_analyze_preset(&analyze.preset) {
        let (paths, rows) = collect_pure_in_memory_rows(
            inputs,
            &scan_opts,
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
        let export_receipt = build_export_receipt(&paths, &scan_opts, &export, data);

        return analyze_with_export_receipt(
            export_receipt,
            logical_inputs,
            PathBuf::new(),
            analyze,
        );
    }

    let scan = tokmd_scan::scan_in_memory(inputs, &scan_opts)?;
    let data = collect_materialized_export_data(&scan, &export);
    let logical_inputs: Vec<String> = scan
        .logical_paths()
        .iter()
        .map(|path| tokmd_model::normalize_path(path, None))
        .collect();
    let root = scan.strip_prefix().to_path_buf();
    let export_receipt = build_export_receipt(scan.logical_paths(), &scan_opts, &export, data);

    analyze_with_export_receipt(export_receipt, logical_inputs, root, analyze)
}

#[doc(hidden)]
pub fn supports_rootless_in_memory_analyze_preset(preset: &str) -> bool {
    let preset = preset.trim();
    preset.eq_ignore_ascii_case("receipt") || preset.eq_ignore_ascii_case("estimate")
}

fn analyze_with_export_receipt(
    export_receipt: ExportReceipt,
    inputs: Vec<String>,
    root: PathBuf,
    analyze: &AnalyzeSettings,
) -> Result<AnalysisReceipt> {
    let request = build_analysis_request(analyze)?;
    let source = AnalysisSource {
        inputs,
        export_path: None,
        base_receipt_path: None,
        export_schema_version: Some(export_receipt.schema_version),
        export_generated_at_ms: Some(export_receipt.generated_at_ms),
        base_signature: None,
        module_roots: export_receipt.data.module_roots.clone(),
        module_depth: export_receipt.data.module_depth,
        children: child_include_mode_to_string(export_receipt.data.children),
    };

    let ctx = analysis::AnalysisContext {
        export: export_receipt.data,
        root,
        source,
    };

    analysis::analyze(ctx, request)
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

fn child_include_mode_to_string(mode: ChildIncludeMode) -> String {
    match mode {
        ChildIncludeMode::Separate => "separate".to_string(),
        ChildIncludeMode::ParentsOnly => "parents-only".to_string(),
    }
}

fn derive_analysis_root(scan: &ScanSettings) -> Option<PathBuf> {
    let first = scan.paths.first()?;
    if first.trim().is_empty() {
        return None;
    }

    let candidate = PathBuf::from(first);
    let absolute = if candidate.is_absolute() {
        candidate
    } else {
        std::env::current_dir().ok()?.join(candidate)
    };

    if absolute.is_dir() {
        Some(absolute)
    } else {
        absolute.parent().map(|p| p.to_path_buf())
    }
}
