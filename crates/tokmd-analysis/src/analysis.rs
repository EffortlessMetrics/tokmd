use std::path::PathBuf;

use anyhow::Result;
use tokmd_analysis_types::{AnalysisArgsMeta, AnalysisLimits, AnalysisReceipt, AnalysisSource};
use tokmd_types::{ExportData, ScanStatus, ToolInfo};

mod enrichers;
mod files;
mod state;

#[cfg(feature = "content")]
use crate::content::{ContentLimits, ImportGranularity as ContentImportGranularity};
use crate::derived::{build_tree, derive_report};
use crate::grid::{PresetKind, PresetPlan, preset_plan_for};
use crate::util::now_ms;
use state::AnalysisOutputs;

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
    pub effort: Option<crate::effort::EffortRequest>,
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
    pub near_dup_scope: tokmd_analysis_types::NearDupScope,
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
    let mut derived = derive_report(&ctx.export, req.window_tokens);
    if req.args.format.contains("tree") {
        derived.tree = Some(build_tree(&ctx.export));
    }

    let mut source = ctx.source.clone();
    if source.base_signature.is_none() {
        source.base_signature = Some(derived.integrity.hash.clone());
    }

    let plan = preset_plan(req.preset);
    let include_git = req.git.unwrap_or(plan.git);
    let has_host_root = files::has_host_root(&ctx.root);
    let files =
        files::collect_required_files(&ctx.root, &req.limits, &plan, has_host_root, &mut warnings);

    let mut outputs = AnalysisOutputs::default();
    enrichers::run_walk_enrichers(
        &ctx.root,
        &plan,
        files.as_deref(),
        &mut outputs,
        &mut warnings,
    );
    enrichers::run_content_enrichers(
        &ctx,
        &req,
        &plan,
        files.as_deref(),
        has_host_root,
        &mut derived,
        &mut outputs,
        &mut warnings,
    );
    enrichers::run_git_enrichers(
        &ctx,
        &req,
        &plan,
        include_git,
        has_host_root,
        &mut outputs,
        &mut warnings,
    );
    enrichers::run_metadata_enrichers(&ctx, &plan, &mut outputs, &mut warnings);
    enrichers::run_fun_enricher(&plan, &derived, &mut outputs, &mut warnings);
    enrichers::run_effort_enricher(&ctx, &req, &derived, &mut outputs, &mut warnings);

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
        effort: outputs.effort,
        fun: outputs.fun,
    })
}
