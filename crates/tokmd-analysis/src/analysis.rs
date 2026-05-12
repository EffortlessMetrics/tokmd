use std::path::PathBuf;

#[cfg(feature = "effort")]
use crate::effort::EffortRequest;
use anyhow::Result;
use tokmd_analysis_types::{AnalysisArgsMeta, AnalysisReceipt, AnalysisSource, NearDupScope};
use tokmd_types::ExportData;

use crate::grid::{PresetKind, PresetPlan, preset_plan_for};

#[path = "analysis/enrichers.rs"]
mod enrichers;
#[path = "analysis/file_inventory.rs"]
mod file_inventory;
#[path = "analysis/receipt.rs"]
mod receipt;
#[path = "analysis/report_set.rs"]
mod report_set;
#[path = "analysis/warnings.rs"]
mod warnings;

/// Canonical preset enum for analysis orchestration.
pub type AnalysisPreset = PresetKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportGranularity {
    Module,
    File,
}

#[derive(Debug, Clone)]
pub struct AnalysisContext {
    pub export: ExportData,
    pub root: PathBuf,
    pub source: AnalysisSource,
}

#[derive(Debug, Clone)]
pub struct AnalysisRequest {
    pub preset: AnalysisPreset,
    pub args: AnalysisArgsMeta,
    pub limits: tokmd_analysis_types::AnalysisLimits,
    #[cfg(feature = "effort")]
    pub effort: Option<EffortRequest>,
    pub window_tokens: Option<usize>,
    pub git: Option<bool>,
    pub import_granularity: ImportGranularity,
    pub detail_functions: bool,
    /// Enable near-duplicate detection.
    pub near_dup: bool,
    /// Near-duplicate similarity threshold (0.0–1.0).
    pub near_dup_threshold: f64,
    /// Maximum files to analyze for near-duplicates.
    pub near_dup_max_files: usize,
    /// Near-duplicate comparison scope.
    pub near_dup_scope: NearDupScope,
    /// Maximum near-duplicate pairs to emit (truncation guardrail).
    pub near_dup_max_pairs: Option<usize>,
    /// Glob patterns to exclude from near-duplicate analysis.
    pub near_dup_exclude: Vec<String>,
}

fn preset_plan(preset: AnalysisPreset) -> PresetPlan {
    preset_plan_for(preset)
}

pub fn analyze(ctx: AnalysisContext, req: AnalysisRequest) -> Result<AnalysisReceipt> {
    let mut warnings = Vec::new();
    let mut derived = receipt::prepare_derived(&ctx.export, &req.args, req.window_tokens);
    let source = receipt::source_with_base_signature(&ctx.source, &derived);
    let plan = preset_plan(req.preset);
    let include_git = req.git.unwrap_or(plan.git);
    let mut reports = report_set::AnalysisReports::default();

    let files = file_inventory::collect_analysis_files(&ctx, &req, &plan, &mut warnings);
    enrichers::run_file_enrichers(
        &ctx,
        &req,
        &plan,
        files.as_deref(),
        &mut derived,
        &mut reports,
        &mut warnings,
    );
    enrichers::run_git_enrichers(&ctx, &req, &plan, include_git, &mut reports, &mut warnings);
    enrichers::run_model_enrichers(&ctx, &plan, &derived, &mut reports, &mut warnings);
    enrichers::run_effort_enricher(&ctx, &req, &derived, &mut reports, &mut warnings);

    Ok(receipt::build_receipt(
        source, req.args, derived, reports, warnings,
    ))
}
