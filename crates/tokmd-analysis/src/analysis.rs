#[cfg(any(feature = "walk", feature = "content", feature = "git"))]
use std::path::Path;
use std::path::PathBuf;

use anyhow::Result;
use tokmd_analysis_types::{
    AnalysisArgsMeta, AnalysisLimits, AnalysisReceipt, AnalysisSource, NearDupScope,
};
use tokmd_types::ExportData;

#[cfg(feature = "content")]
use crate::content::{ContentLimits, ImportGranularity as ContentImportGranularity};
#[cfg(feature = "effort")]
use crate::effort::EffortRequest;
use crate::grid::{PresetKind, PresetPlan, preset_plan_for};

mod content_reports;
mod derived_stage;
mod file_inventory;
mod fun_effort_reports;
mod git_reports;
mod identity_reports;
mod quality_reports;
mod receipt;
mod reports;
mod walk_reports;

use reports::AnalysisReports;

/// Canonical preset enum for analysis orchestration.
pub type AnalysisPreset = PresetKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportGranularity {
    Module,
    File,
}

#[cfg(feature = "content")]
fn content_limits(limits: &AnalysisLimits) -> ContentLimits {
    ContentLimits {
        max_bytes: limits.max_bytes,
        max_file_bytes: limits.max_file_bytes,
    }
}

#[cfg(feature = "content")]
fn content_import_granularity(granularity: ImportGranularity) -> ContentImportGranularity {
    match granularity {
        ImportGranularity::Module => ContentImportGranularity::Module,
        ImportGranularity::File => ContentImportGranularity::File,
    }
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
    pub limits: AnalysisLimits,
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

#[cfg(any(feature = "walk", feature = "content", feature = "git"))]
const ROOTLESS_FILE_ANALYSIS_WARNING: &str =
    "in-memory analysis has no host root; skipping file-backed enrichers";
#[cfg(feature = "git")]
const ROOTLESS_GIT_ANALYSIS_WARNING: &str =
    "in-memory analysis has no host root; skipping git-backed enrichers";

#[cfg(any(feature = "walk", feature = "content", feature = "git"))]
fn has_host_root(root: &Path) -> bool {
    !root.as_os_str().is_empty()
}

#[cfg(any(feature = "walk", feature = "content", feature = "git"))]
fn push_warning_once(warnings: &mut Vec<String>, warning: &str) {
    if warnings.iter().all(|existing| existing != warning) {
        warnings.push(warning.to_string());
    }
}

pub fn analyze(ctx: AnalysisContext, req: AnalysisRequest) -> Result<AnalysisReceipt> {
    let mut warnings = Vec::new();
    let mut derived = derived_stage::build_derived_report(&ctx.export, &req);
    let source = derived_stage::source_with_base_signature(&ctx.source, &derived);

    let plan = preset_plan(req.preset);
    let include_git = req.git.unwrap_or(plan.git);
    let files =
        file_inventory::collect_files_for_plan(&ctx.root, &req.limits, &plan, &mut warnings);
    let file_slice = files.as_deref();

    let mut reports = AnalysisReports::default();
    walk_reports::enrich_walk_reports(&ctx.root, file_slice, &plan, &mut reports, &mut warnings);
    content_reports::enrich_content_reports(
        content_reports::ContentEnrichmentContext {
            root: &ctx.root,
            export: &ctx.export,
            files: file_slice,
            plan: &plan,
            req: &req,
        },
        &mut derived,
        &mut reports,
        &mut warnings,
    );
    git_reports::enrich_git_reports(
        &ctx.root,
        &ctx.export,
        &req.limits,
        include_git,
        &plan,
        &mut reports,
        &mut warnings,
    );
    identity_reports::enrich_identity_reports(&ctx.export, &plan, &mut reports, &mut warnings);
    quality_reports::enrich_quality_reports(
        &ctx.root,
        &ctx.export,
        file_slice,
        &plan,
        &req,
        &mut reports,
        &mut warnings,
    );
    fun_effort_reports::enrich_fun_report(&plan, &derived, &mut reports, &mut warnings);

    #[cfg(feature = "effort")]
    fun_effort_reports::enrich_effort_report(
        &ctx.root,
        &ctx.export,
        &derived,
        req.effort.as_ref(),
        &mut reports,
        &mut warnings,
    );

    Ok(receipt::build_receipt(
        source, req.args, derived, reports, warnings,
    ))
}
