use std::path::PathBuf;

use anyhow::Result;
use tokmd_analysis_types::{AnalysisArgsMeta, AnalysisReceipt, AnalysisSource, NearDupScope};
use tokmd_types::{ExportData, ScanStatus, ToolInfo};

#[cfg(feature = "effort")]
use crate::effort::EffortRequest;
use crate::grid::{PresetKind, PresetPlan, preset_plan_for};
use crate::util::now_ms;

mod enrichers;
mod files;
mod outputs;
mod setup;

use outputs::AnalysisOutputs;

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
    let mut derived = setup::build_derived(&ctx.export, &req);
    let source = setup::source_with_signature(ctx.source, derived.integrity.hash.clone());

    let plan = preset_plan(req.preset);
    let include_git = req.git.unwrap_or(plan.git);
    let has_host_root = files::has_host_root(&ctx.root);
    let files = files::collect_required_files(
        &ctx.root,
        &plan,
        req.limits.max_files,
        has_host_root,
        &mut warnings,
    );
    let file_slice = files.as_deref();

    let mut outputs = AnalysisOutputs::default();
    enrichers::inventory::run(&ctx.root, file_slice, &plan, &mut outputs, &mut warnings);
    enrichers::content::run(
        enrichers::content::ContentInput {
            root: &ctx.root,
            export: &ctx.export,
            files: file_slice,
            plan: &plan,
            req: &req,
            has_host_root,
        },
        &mut derived,
        &mut outputs,
        &mut warnings,
    );
    enrichers::git::run(
        enrichers::git::GitInput {
            root: &ctx.root,
            export: &ctx.export,
            plan: &plan,
            include_git,
            max_commits: req.limits.max_commits,
            max_commit_files: req.limits.max_commit_files,
            has_host_root,
        },
        &mut outputs,
        &mut warnings,
    );
    enrichers::semantic::run(&ctx.export, &derived, &plan, &mut outputs, &mut warnings);
    enrichers::code_quality::run(
        enrichers::code_quality::CodeQualityInput {
            root: &ctx.root,
            export: &ctx.export,
            files: file_slice,
            plan: &plan,
            limits: &req.limits,
            detail_functions: req.detail_functions,
        },
        &mut outputs,
        &mut warnings,
    );

    #[cfg(feature = "effort")]
    let effort = enrichers::effort::run(
        &ctx.root,
        &ctx.export,
        &derived,
        &outputs,
        req.effort.as_ref(),
        &mut warnings,
    );
    #[cfg(not(feature = "effort"))]
    let effort: Option<tokmd_analysis_types::EffortEstimateReport> = None;

    let status = if warnings.is_empty() {
        ScanStatus::Complete
    } else {
        ScanStatus::Partial
    };

    Ok(AnalysisReceipt {
        schema_version: tokmd_analysis_types::ANALYSIS_SCHEMA_VERSION,
        generated_at_ms: now_ms(),
        tool: ToolInfo::current(),
        mode: "analysis".to_string(),
        status,
        warnings,
        source,
        args: req.args,
        archetype: outputs.archetype,
        topics: outputs.topics,
        entropy: outputs.entropy,
        predictive_churn: outputs.churn,
        corporate_fingerprint: outputs.fingerprint,
        license: outputs.license,
        derived: Some(derived),
        assets: outputs.assets,
        deps: outputs.deps,
        git: outputs.git,
        imports: outputs.imports,
        dup: outputs.dup,
        complexity: outputs.complexity,
        api_surface: outputs.api_surface,
        effort,
        fun: outputs.fun,
    })
}
