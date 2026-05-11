//! Analysis workflow facade.

use std::path::PathBuf;

use anyhow::Result;
use tokmd_analysis as analysis;
use tokmd_analysis_types::{AnalysisArgsMeta, AnalysisReceipt, AnalysisSource};
use tokmd_settings::ScanOptions;
use tokmd_types::{ChildIncludeMode, ExportData, ExportReceipt, FileRow};

use crate::settings::{AnalyzeSettings, ExportSettings, ScanSettings};
use crate::{InMemoryFile, build_export_receipt, error};

use super::{
    collect_pure_in_memory_rows, deterministic_in_memory_scan_options, strip_virtual_export_prefix,
};

use super::export_workflow;

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

fn build_analysis_request(analyze: &AnalyzeSettings) -> Result<analysis::AnalysisRequest> {
    let (preset, preset_meta) = parse_analysis_preset(&analyze.preset)?;
    let (granularity, granularity_meta) = parse_import_granularity(&analyze.granularity)?;
    let effort = parse_effort_request(analyze, &preset_meta)?;

    Ok(analysis::AnalysisRequest {
        preset,
        args: AnalysisArgsMeta {
            preset: preset_meta,
            format: "json".to_string(),
            window_tokens: analyze.window,
            git: analyze.git,
            max_files: analyze.max_files,
            max_bytes: analyze.max_bytes,
            max_file_bytes: analyze.max_file_bytes,
            max_commits: analyze.max_commits,
            max_commit_files: analyze.max_commit_files,
            import_granularity: granularity_meta,
        },
        limits: analysis::AnalysisLimits {
            max_files: analyze.max_files,
            max_bytes: analyze.max_bytes,
            max_file_bytes: analyze.max_file_bytes,
            max_commits: analyze.max_commits,
            max_commit_files: analyze.max_commit_files,
        },
        window_tokens: analyze.window,
        git: analyze.git,
        import_granularity: granularity,
        detail_functions: false,
        near_dup: false,
        near_dup_threshold: 0.80,
        near_dup_max_files: 2000,
        near_dup_scope: analysis::NearDupScope::Module,
        near_dup_max_pairs: None,
        near_dup_exclude: Vec::new(),
        effort,
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

pub(crate) fn parse_analysis_preset(value: &str) -> Result<(analysis::AnalysisPreset, String)> {
    let normalized = value.trim().to_ascii_lowercase();
    let preset = match normalized.as_str() {
        "receipt" => analysis::AnalysisPreset::Receipt,
        "estimate" => analysis::AnalysisPreset::Estimate,
        "health" => analysis::AnalysisPreset::Health,
        "risk" => analysis::AnalysisPreset::Risk,
        "supply" => analysis::AnalysisPreset::Supply,
        "architecture" => analysis::AnalysisPreset::Architecture,
        "topics" => analysis::AnalysisPreset::Topics,
        "security" => analysis::AnalysisPreset::Security,
        "identity" => analysis::AnalysisPreset::Identity,
        "git" => analysis::AnalysisPreset::Git,
        "deep" => analysis::AnalysisPreset::Deep,
        "fun" => analysis::AnalysisPreset::Fun,
        _ => {
            return Err(error::TokmdError::invalid_field(
                "preset",
                "'receipt', 'estimate', 'health', 'risk', 'supply', 'architecture', 'topics', 'security', 'identity', 'git', 'deep', or 'fun'",
            )
            .into());
        }
    };
    Ok((preset, normalized))
}

fn parse_import_granularity(value: &str) -> Result<(analysis::ImportGranularity, String)> {
    let normalized = value.trim().to_ascii_lowercase();
    let granularity = match normalized.as_str() {
        "module" => analysis::ImportGranularity::Module,
        "file" => analysis::ImportGranularity::File,
        _ => {
            return Err(
                error::TokmdError::invalid_field("granularity", "'module' or 'file'").into(),
            );
        }
    };
    Ok((granularity, normalized))
}

pub(crate) fn parse_effort_request(
    analyze: &AnalyzeSettings,
    preset: &str,
) -> Result<Option<analysis::EffortRequest>> {
    let request = analysis::EffortRequest::default();
    let requested = preset == "estimate"
        || analyze.effort_model.is_some()
        || analyze.effort_layer.is_some()
        || analyze.effort_base_ref.is_some()
        || analyze.effort_head_ref.is_some()
        || analyze.effort_monte_carlo.unwrap_or(false)
        || analyze.effort_mc_iterations.is_some()
        || analyze.effort_mc_seed.is_some();

    if !requested {
        return Ok(None);
    }

    if (analyze.effort_base_ref.is_some() && analyze.effort_head_ref.is_none())
        || (analyze.effort_base_ref.is_none() && analyze.effort_head_ref.is_some())
    {
        return Err(error::TokmdError::invalid_field(
            "effort_base_ref/effort_head_ref",
            "both effort_base_ref and effort_head_ref must be provided together",
        )
        .into());
    }

    let model = analyze
        .effort_model
        .as_deref()
        .map(parse_effort_model)
        .transpose()?
        .unwrap_or(request.model);
    let layer = analyze
        .effort_layer
        .as_deref()
        .map(parse_effort_layer)
        .transpose()?
        .unwrap_or(request.layer);

    let monte_carlo = analyze.effort_monte_carlo.unwrap_or(false);

    let mc_iterations = analyze
        .effort_mc_iterations
        .unwrap_or(request.mc_iterations);

    if mc_iterations == 0 {
        return Err(error::TokmdError::invalid_field(
            "effort_mc_iterations",
            "must be greater than 0",
        )
        .into());
    }

    Ok(Some(analysis::EffortRequest {
        model,
        layer,
        base_ref: analyze.effort_base_ref.clone(),
        head_ref: analyze.effort_head_ref.clone(),
        monte_carlo,
        mc_iterations,
        mc_seed: analyze.effort_mc_seed,
    }))
}

fn parse_effort_model(value: &str) -> Result<analysis::EffortModelKind> {
    match value.trim().to_ascii_lowercase().as_str() {
        "cocomo81-basic" => Ok(analysis::EffortModelKind::Cocomo81Basic),
        "cocomo2-early" | "ensemble" => Err(error::TokmdError::invalid_field(
            "effort_model",
            "only 'cocomo81-basic' is currently supported",
        )
        .into()),
        _ => Err(error::TokmdError::invalid_field("effort_model", "'cocomo81-basic'").into()),
    }
}

fn parse_effort_layer(value: &str) -> Result<analysis::EffortLayer> {
    match value.trim().to_ascii_lowercase().as_str() {
        "headline" => Ok(analysis::EffortLayer::Headline),
        "why" => Ok(analysis::EffortLayer::Why),
        "full" => Ok(analysis::EffortLayer::Full),
        _ => Err(
            error::TokmdError::invalid_field("effort_layer", "'headline', 'why', or 'full'").into(),
        ),
    }
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
