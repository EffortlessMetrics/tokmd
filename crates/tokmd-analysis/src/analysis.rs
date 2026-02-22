use std::path::PathBuf;

use anyhow::Result;
use tokmd_analysis_grid::{PresetKind, PresetPlan, preset_plan_for};
use tokmd_analysis_types::{
    AnalysisArgsMeta, AnalysisReceipt, AnalysisSource, ApiSurfaceReport, Archetype, AssetReport,
    ComplexityReport, CorporateFingerprint, DependencyReport, DuplicateReport, EntropyReport,
    FunReport, GitReport, ImportReport, LicenseReport, NearDupScope, PredictiveChurnReport,
    TopicClouds,
};
use tokmd_analysis_util::AnalysisLimits;
use tokmd_types::{ExportData, ScanStatus, ToolInfo};

#[cfg(feature = "git")]
use crate::churn::build_predictive_churn_report;
#[cfg(feature = "content")]
use crate::content::{build_duplicate_report, build_import_report, build_todo_report};
use crate::derived::{build_tree, derive_report};
#[cfg(feature = "git")]
use crate::git::build_git_report;
use crate::util::now_ms;
#[cfg(all(feature = "content", feature = "walk"))]
use tokmd_analysis_api_surface::build_api_surface_report;
#[cfg(feature = "archetype")]
use tokmd_analysis_archetype::detect_archetype;
#[cfg(feature = "walk")]
use tokmd_analysis_assets::{build_assets_report, build_dependency_report};
#[cfg(all(feature = "content", feature = "walk"))]
use tokmd_analysis_complexity::build_complexity_report;
#[cfg(all(feature = "content", feature = "walk"))]
use tokmd_analysis_entropy::build_entropy_report;
#[cfg(feature = "git")]
use tokmd_analysis_fingerprint::build_corporate_fingerprint;
#[cfg(feature = "fun")]
use tokmd_analysis_fun::build_fun_report;
#[cfg(all(feature = "halstead", feature = "content", feature = "walk"))]
use tokmd_analysis_halstead::build_halstead_report;
#[cfg(all(feature = "content", feature = "walk"))]
use tokmd_analysis_license::build_license_report;
#[cfg(feature = "content")]
use tokmd_analysis_near_dup::{NearDupLimits, build_near_dup_report};
#[cfg(feature = "topics")]
use tokmd_analysis_topics::build_topic_clouds;

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
    pub limits: AnalysisLimits,
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
    let mut warnings: Vec<String> = Vec::new();
    #[cfg_attr(not(feature = "content"), allow(unused_mut))]
    let mut derived = derive_report(&ctx.export, req.window_tokens);
    if req.args.format.contains("tree") {
        derived.tree = Some(build_tree(&ctx.export));
    }

    let mut source = ctx.source.clone();
    if source.base_signature.is_none() {
        source.base_signature = Some(derived.integrity.hash.clone());
    }

    let plan = preset_plan(req.preset);
    let include_git = match req.git {
        Some(flag) => flag,
        None => plan.git,
    };

    #[cfg(feature = "walk")]
    let mut assets: Option<AssetReport> = None;
    #[cfg(not(feature = "walk"))]
    let assets: Option<AssetReport> = None;

    #[cfg(feature = "walk")]
    let mut deps: Option<DependencyReport> = None;
    #[cfg(not(feature = "walk"))]
    let deps: Option<DependencyReport> = None;

    #[cfg(feature = "content")]
    let mut imports: Option<ImportReport> = None;
    #[cfg(not(feature = "content"))]
    let imports: Option<ImportReport> = None;

    #[cfg(feature = "content")]
    let mut dup: Option<DuplicateReport> = None;
    #[cfg(not(feature = "content"))]
    let dup: Option<DuplicateReport> = None;

    #[cfg(feature = "git")]
    let mut git: Option<GitReport> = None;
    #[cfg(not(feature = "git"))]
    let git: Option<GitReport> = None;

    #[cfg(feature = "git")]
    let mut churn: Option<PredictiveChurnReport> = None;
    #[cfg(not(feature = "git"))]
    let churn: Option<PredictiveChurnReport> = None;

    #[cfg(feature = "git")]
    let mut fingerprint: Option<CorporateFingerprint> = None;
    #[cfg(not(feature = "git"))]
    let fingerprint: Option<CorporateFingerprint> = None;

    #[cfg(all(feature = "content", feature = "walk"))]
    let mut entropy: Option<EntropyReport> = None;
    #[cfg(not(all(feature = "content", feature = "walk")))]
    let entropy: Option<EntropyReport> = None;

    #[cfg(all(feature = "content", feature = "walk"))]
    let mut license: Option<LicenseReport> = None;
    #[cfg(not(all(feature = "content", feature = "walk")))]
    let license: Option<LicenseReport> = None;

    #[cfg(all(feature = "content", feature = "walk"))]
    let mut complexity: Option<ComplexityReport> = None;
    #[cfg(not(all(feature = "content", feature = "walk")))]
    let complexity: Option<ComplexityReport> = None;

    #[cfg(all(feature = "content", feature = "walk"))]
    let mut api_surface: Option<ApiSurfaceReport> = None;
    #[cfg(not(all(feature = "content", feature = "walk")))]
    let api_surface: Option<ApiSurfaceReport> = None;

    #[cfg(feature = "archetype")]
    let mut archetype: Option<Archetype> = None;
    #[cfg(not(feature = "archetype"))]
    let archetype: Option<Archetype> = None;
    #[cfg(feature = "topics")]
    let mut topics: Option<TopicClouds> = None;
    #[cfg(not(feature = "topics"))]
    let topics: Option<TopicClouds> = None;

    let fun: Option<FunReport>;

    #[cfg(any(feature = "walk", feature = "content"))]
    let mut files: Option<Vec<PathBuf>> = None;
    #[cfg(not(any(feature = "walk", feature = "content")))]
    let _files: Option<Vec<PathBuf>> = None;

    if plan.needs_files() {
        #[cfg(feature = "walk")]
        match tokmd_walk::list_files(&ctx.root, req.limits.max_files) {
            Ok(list) => files = Some(list),
            Err(err) => warnings.push(format!("walk failed: {}", err)),
        }
        #[cfg(not(feature = "walk"))]
        {
            warnings.push(
                tokmd_analysis_grid::DisabledFeature::FileInventory
                    .warning()
                    .to_string(),
            );
        }
    }

    if plan.assets {
        #[cfg(feature = "walk")]
        {
            if let Some(list) = files.as_deref() {
                match build_assets_report(&ctx.root, list) {
                    Ok(report) => assets = Some(report),
                    Err(err) => warnings.push(format!("asset scan failed: {}", err)),
                }
            }
        }
    }

    if plan.deps {
        #[cfg(feature = "walk")]
        {
            if let Some(list) = files.as_deref() {
                match build_dependency_report(&ctx.root, list) {
                    Ok(report) => deps = Some(report),
                    Err(err) => warnings.push(format!("dependency scan failed: {}", err)),
                }
            }
        }
    }

    if plan.todo {
        #[cfg(feature = "content")]
        {
            if let Some(list) = files.as_deref() {
                match build_todo_report(&ctx.root, list, &req.limits, derived.totals.code) {
                    Ok(report) => derived.todo = Some(report),
                    Err(err) => warnings.push(format!("todo scan failed: {}", err)),
                }
            }
        }
        #[cfg(not(feature = "content"))]
        warnings.push(
            tokmd_analysis_grid::DisabledFeature::TodoScan
                .warning()
                .to_string(),
        );
    }

    if plan.dup {
        #[cfg(feature = "content")]
        {
            if let Some(list) = files.as_deref() {
                match build_duplicate_report(&ctx.root, list, &ctx.export, &req.limits) {
                    Ok(report) => dup = Some(report),
                    Err(err) => warnings.push(format!("dup scan failed: {}", err)),
                }
            }
        }
        #[cfg(not(feature = "content"))]
        warnings.push(
            tokmd_analysis_grid::DisabledFeature::DuplicationScan
                .warning()
                .to_string(),
        );
    }

    // Near-duplicate detection (opt-in via --near-dup)
    if req.near_dup {
        #[cfg(feature = "content")]
        {
            let near_dup_limits = NearDupLimits {
                max_bytes: req.limits.max_bytes,
                max_file_bytes: req.limits.max_file_bytes,
            };
            match build_near_dup_report(
                &ctx.root,
                &ctx.export,
                req.near_dup_scope,
                req.near_dup_threshold,
                req.near_dup_max_files,
                req.near_dup_max_pairs,
                &near_dup_limits,
                &req.near_dup_exclude,
            ) {
                Ok(report) => {
                    // Attach to existing dup report or create a minimal one
                    if let Some(ref mut d) = dup {
                        d.near = Some(report);
                    } else {
                        dup = Some(DuplicateReport {
                            groups: Vec::new(),
                            wasted_bytes: 0,
                            strategy: "none".to_string(),
                            density: None,
                            near: Some(report),
                        });
                    }
                }
                Err(err) => warnings.push(format!("near-dup scan failed: {}", err)),
            }
        }
        #[cfg(not(feature = "content"))]
        warnings.push(
            tokmd_analysis_grid::DisabledFeature::NearDuplicateScan
                .warning()
                .to_string(),
        );
    }

    if plan.imports {
        #[cfg(feature = "content")]
        {
            if let Some(list) = files.as_deref() {
                match build_import_report(
                    &ctx.root,
                    list,
                    &ctx.export,
                    req.import_granularity,
                    &req.limits,
                ) {
                    Ok(report) => imports = Some(report),
                    Err(err) => warnings.push(format!("import scan failed: {}", err)),
                }
            }
        }
        #[cfg(not(feature = "content"))]
        warnings.push(
            tokmd_analysis_grid::DisabledFeature::ImportScan
                .warning()
                .to_string(),
        );
    }

    if include_git {
        #[cfg(feature = "git")]
        {
            let repo_root = match tokmd_git::repo_root(&ctx.root) {
                Some(root) => root,
                None => {
                    warnings.push("git scan failed: not a git repo".to_string());
                    PathBuf::new()
                }
            };
            if !repo_root.as_os_str().is_empty() {
                match tokmd_git::collect_history(
                    &repo_root,
                    req.limits.max_commits,
                    req.limits.max_commit_files,
                ) {
                    Ok(commits) => {
                        if plan.git {
                            match build_git_report(&repo_root, &ctx.export, &commits) {
                                Ok(report) => git = Some(report),
                                Err(err) => warnings.push(format!("git scan failed: {}", err)),
                            }
                        }
                        if plan.churn {
                            churn = Some(build_predictive_churn_report(
                                &ctx.export,
                                &commits,
                                &repo_root,
                            ));
                        }
                        if plan.fingerprint {
                            fingerprint = Some(build_corporate_fingerprint(&commits));
                        }
                    }
                    Err(err) => warnings.push(format!("git scan failed: {}", err)),
                }
            }
        }
        #[cfg(not(feature = "git"))]
        warnings.push(
            tokmd_analysis_grid::DisabledFeature::GitMetrics
                .warning()
                .to_string(),
        );
    }

    if plan.archetype {
        #[cfg(feature = "archetype")]
        {
            archetype = detect_archetype(&ctx.export);
        }
        #[cfg(not(feature = "archetype"))]
        {
            warnings.push(
                tokmd_analysis_grid::DisabledFeature::Archetype
                    .warning()
                    .to_string(),
            );
        }
    }

    if plan.topics {
        #[cfg(feature = "topics")]
        {
            topics = Some(build_topic_clouds(&ctx.export));
        }
        #[cfg(not(feature = "topics"))]
        {
            warnings.push(
                tokmd_analysis_grid::DisabledFeature::Topics
                    .warning()
                    .to_string(),
            );
        }
    }

    if plan.entropy {
        #[cfg(all(feature = "content", feature = "walk"))]
        {
            if let Some(list) = files.as_deref() {
                match build_entropy_report(&ctx.root, list, &ctx.export, &req.limits) {
                    Ok(report) => entropy = Some(report),
                    Err(err) => warnings.push(format!("entropy scan failed: {}", err)),
                }
            }
        }
        #[cfg(not(all(feature = "content", feature = "walk")))]
        warnings.push(
            tokmd_analysis_grid::DisabledFeature::EntropyProfiling
                .warning()
                .to_string(),
        );
    }

    if plan.license {
        #[cfg(all(feature = "content", feature = "walk"))]
        {
            if let Some(list) = files.as_deref() {
                match build_license_report(&ctx.root, list, &req.limits) {
                    Ok(report) => license = Some(report),
                    Err(err) => warnings.push(format!("license scan failed: {}", err)),
                }
            }
        }
        #[cfg(not(all(feature = "content", feature = "walk")))]
        warnings.push(
            tokmd_analysis_grid::DisabledFeature::LicenseRadar
                .warning()
                .to_string(),
        );
    }

    if plan.complexity {
        #[cfg(all(feature = "content", feature = "walk"))]
        {
            if let Some(list) = files.as_deref() {
                match build_complexity_report(
                    &ctx.root,
                    list,
                    &ctx.export,
                    &req.limits,
                    req.detail_functions,
                ) {
                    Ok(report) => complexity = Some(report),
                    Err(err) => warnings.push(format!("complexity scan failed: {}", err)),
                }
            }
        }
        #[cfg(not(all(feature = "content", feature = "walk")))]
        warnings.push(
            tokmd_analysis_grid::DisabledFeature::ComplexityAnalysis
                .warning()
                .to_string(),
        );
    }

    if plan.api_surface {
        #[cfg(all(feature = "content", feature = "walk"))]
        {
            if let Some(list) = files.as_deref() {
                match build_api_surface_report(&ctx.root, list, &ctx.export, &req.limits) {
                    Ok(report) => api_surface = Some(report),
                    Err(err) => warnings.push(format!("api surface scan failed: {}", err)),
                }
            }
        }
        #[cfg(not(all(feature = "content", feature = "walk")))]
        warnings.push(
            tokmd_analysis_grid::DisabledFeature::ApiSurfaceAnalysis
                .warning()
                .to_string(),
        );
    }

    // Halstead metrics (feature-gated)
    #[cfg(all(feature = "halstead", feature = "content", feature = "walk"))]
    if plan.halstead
        && let Some(list) = files.as_deref()
    {
        match build_halstead_report(&ctx.root, list, &ctx.export, &req.limits) {
            Ok(halstead_report) => {
                // Wire Halstead into complexity report if available
                if let Some(ref mut cx) = complexity {
                    // Update maintainability index with Halstead volume
                    if let Some(ref mut mi) = cx.maintainability_index {
                        let vol = halstead_report.volume;
                        if vol > 0.0 {
                            mi.avg_halstead_volume = Some(vol);
                            // Recompute with full SEI formula
                            let score = (171.0
                                - 5.2 * vol.ln()
                                - 0.23 * mi.avg_cyclomatic
                                - 16.2 * mi.avg_loc.ln())
                            .max(0.0);
                            let factor = 100.0;
                            mi.score = (score * factor).round() / factor;
                            mi.grade = if mi.score >= 85.0 {
                                "A".to_string()
                            } else if mi.score >= 65.0 {
                                "B".to_string()
                            } else {
                                "C".to_string()
                            };
                        }
                    }
                    cx.halstead = Some(halstead_report);
                }
            }
            Err(err) => warnings.push(format!("halstead scan failed: {}", err)),
        }
    }

    if plan.fun {
        #[cfg(feature = "fun")]
        {
            fun = Some(build_fun_report(&derived));
        }
        #[cfg(not(feature = "fun"))]
        {
            warnings.push(
                tokmd_analysis_grid::DisabledFeature::Fun
                    .warning()
                    .to_string(),
            );
            fun = None;
        }
    } else {
        fun = None;
    }

    let status = if warnings.is_empty() {
        ScanStatus::Complete
    } else {
        ScanStatus::Partial
    };

    let receipt = AnalysisReceipt {
        schema_version: tokmd_analysis_types::ANALYSIS_SCHEMA_VERSION,
        generated_at_ms: now_ms(),
        tool: ToolInfo::current(),
        mode: "analysis".to_string(),
        status,
        warnings,
        source,
        args: req.args,
        archetype,
        topics,
        entropy,
        predictive_churn: churn,
        corporate_fingerprint: fingerprint,
        license,
        derived: Some(derived),
        assets,
        deps,
        git,
        imports,
        dup,
        complexity,
        api_surface,
        fun,
    };

    Ok(receipt)
}

