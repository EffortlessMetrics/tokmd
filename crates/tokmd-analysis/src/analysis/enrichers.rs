#![allow(
    clippy::ptr_arg,
    clippy::too_many_arguments,
    dead_code,
    unused_imports,
    unused_variables
)]

use std::path::{Path, PathBuf};

#[cfg(all(feature = "content", feature = "walk"))]
use crate::api_surface::build_api_surface_report;
#[cfg(feature = "archetype")]
use crate::archetype::detect_archetype;
#[cfg(feature = "walk")]
use crate::assets::{build_assets_report, build_dependency_report};
#[cfg(feature = "content")]
use crate::content::{build_duplicate_report, build_import_report, build_todo_report};
#[cfg(feature = "effort")]
use crate::effort::build_effort_report;
#[cfg(feature = "git")]
use crate::fingerprint::build_corporate_fingerprint;
#[cfg(feature = "fun")]
use crate::fun::build_fun_report;
#[cfg(feature = "git")]
use crate::git::{build_git_report, build_predictive_churn_report};
use crate::grid::PresetPlan;
#[cfg(feature = "content")]
use crate::near_dup::{NearDupLimits, build_near_dup_report};
#[cfg(feature = "topics")]
use crate::topics::build_topic_clouds;
#[cfg(all(feature = "content", feature = "walk"))]
use crate::{
    complexity::build_complexity_report, entropy::build_entropy_report,
    license::build_license_report,
};
#[cfg(all(feature = "halstead", feature = "content", feature = "walk"))]
use crate::{halstead::build_halstead_report, maintainability::attach_halstead_metrics};
use tokmd_analysis_types::{DerivedReport, DuplicateReport};

use super::files::{ROOTLESS_GIT_ANALYSIS_WARNING, push_warning_once};
use super::{AnalysisContext, AnalysisRequest, state::AnalysisOutputs};
#[cfg(feature = "content")]
use super::{content_import_granularity, content_limits};

pub(super) fn run_walk_enrichers(
    root: &Path,
    plan: &PresetPlan,
    files: Option<&[PathBuf]>,
    outputs: &mut AnalysisOutputs,
    warnings: &mut Vec<String>,
) {
    if plan.assets {
        #[cfg(feature = "walk")]
        if let Some(list) = files {
            match build_assets_report(root, list) {
                Ok(report) => outputs.assets = Some(report),
                Err(err) => warnings.push(format!("asset scan failed: {}", err)),
            }
        }
    }

    if plan.deps {
        #[cfg(feature = "walk")]
        if let Some(list) = files {
            match build_dependency_report(root, list) {
                Ok(report) => outputs.deps = Some(report),
                Err(err) => warnings.push(format!("dependency scan failed: {}", err)),
            }
        }
    }
}

pub(super) fn run_content_enrichers(
    ctx: &AnalysisContext,
    req: &AnalysisRequest,
    plan: &PresetPlan,
    files: Option<&[PathBuf]>,
    has_host_root: bool,
    derived: &mut DerivedReport,
    outputs: &mut AnalysisOutputs,
    warnings: &mut Vec<String>,
) {
    run_todo(ctx, req, plan, files, derived, warnings);
    run_duplicate(ctx, req, plan, files, has_host_root, outputs, warnings);
    run_imports(ctx, req, plan, files, outputs, warnings);
    run_file_backed_reports(ctx, req, plan, files, outputs, warnings);
}

fn run_todo(
    ctx: &AnalysisContext,
    req: &AnalysisRequest,
    plan: &PresetPlan,
    files: Option<&[PathBuf]>,
    derived: &mut DerivedReport,
    warnings: &mut Vec<String>,
) {
    if plan.todo {
        #[cfg(feature = "content")]
        if let Some(list) = files {
            let limits = content_limits(&req.limits);
            match build_todo_report(&ctx.root, list, &limits, derived.totals.code) {
                Ok(report) => derived.todo = Some(report),
                Err(err) => warnings.push(format!("todo scan failed: {}", err)),
            }
        }
        #[cfg(not(feature = "content"))]
        warnings.push(crate::grid::DisabledFeature::TodoScan.warning().to_string());
    }
}

fn run_duplicate(
    ctx: &AnalysisContext,
    req: &AnalysisRequest,
    plan: &PresetPlan,
    files: Option<&[PathBuf]>,
    has_host_root: bool,
    outputs: &mut AnalysisOutputs,
    warnings: &mut Vec<String>,
) {
    if plan.dup {
        #[cfg(feature = "content")]
        if let Some(list) = files {
            let limits = content_limits(&req.limits);
            match build_duplicate_report(&ctx.root, list, &ctx.export, &limits) {
                Ok(report) => outputs.dup = Some(report),
                Err(err) => warnings.push(format!("dup scan failed: {}", err)),
            }
        }
        #[cfg(not(feature = "content"))]
        warnings.push(
            crate::grid::DisabledFeature::DuplicationScan
                .warning()
                .to_string(),
        );
    }

    if req.near_dup {
        run_near_duplicate(ctx, req, has_host_root, outputs, warnings);
    }
}

fn run_near_duplicate(
    ctx: &AnalysisContext,
    req: &AnalysisRequest,
    has_host_root: bool,
    outputs: &mut AnalysisOutputs,
    warnings: &mut Vec<String>,
) {
    #[cfg(feature = "content")]
    {
        if has_host_root {
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
                Ok(report) => attach_near_duplicate(outputs, report),
                Err(err) => warnings.push(format!("near-dup scan failed: {}", err)),
            }
        } else {
            push_warning_once(
                warnings,
                "in-memory analysis has no host root; skipping file-backed enrichers",
            );
        }
    }
    #[cfg(not(feature = "content"))]
    warnings.push(
        crate::grid::DisabledFeature::NearDuplicateScan
            .warning()
            .to_string(),
    );
}

#[cfg(feature = "content")]
fn attach_near_duplicate(
    outputs: &mut AnalysisOutputs,
    report: tokmd_analysis_types::NearDuplicateReport,
) {
    if let Some(ref mut duplicate) = outputs.dup {
        duplicate.near = Some(report);
    } else {
        outputs.dup = Some(DuplicateReport {
            groups: Vec::new(),
            wasted_bytes: 0,
            strategy: "none".to_string(),
            density: None,
            near: Some(report),
        });
    }
}

fn run_imports(
    ctx: &AnalysisContext,
    req: &AnalysisRequest,
    plan: &PresetPlan,
    files: Option<&[PathBuf]>,
    outputs: &mut AnalysisOutputs,
    warnings: &mut Vec<String>,
) {
    if plan.imports {
        #[cfg(feature = "content")]
        if let Some(list) = files {
            let limits = content_limits(&req.limits);
            let granularity = content_import_granularity(req.import_granularity);
            match build_import_report(&ctx.root, list, &ctx.export, granularity, &limits) {
                Ok(report) => outputs.imports = Some(report),
                Err(err) => warnings.push(format!("import scan failed: {}", err)),
            }
        }
        #[cfg(not(feature = "content"))]
        warnings.push(
            crate::grid::DisabledFeature::ImportScan
                .warning()
                .to_string(),
        );
    }
}

fn run_file_backed_reports(
    ctx: &AnalysisContext,
    req: &AnalysisRequest,
    plan: &PresetPlan,
    files: Option<&[PathBuf]>,
    outputs: &mut AnalysisOutputs,
    warnings: &mut Vec<String>,
) {
    run_entropy(ctx, req, plan, files, outputs, warnings);
    run_license(ctx, req, plan, files, outputs, warnings);
    run_complexity(ctx, req, plan, files, outputs, warnings);
    run_api_surface(ctx, req, plan, files, outputs, warnings);
    run_halstead(ctx, req, plan, files, outputs, warnings);
}

fn run_entropy(
    ctx: &AnalysisContext,
    req: &AnalysisRequest,
    plan: &PresetPlan,
    files: Option<&[PathBuf]>,
    outputs: &mut AnalysisOutputs,
    warnings: &mut Vec<String>,
) {
    if plan.entropy {
        #[cfg(all(feature = "content", feature = "walk"))]
        if let Some(list) = files {
            match build_entropy_report(&ctx.root, list, &ctx.export, &req.limits) {
                Ok(report) => outputs.entropy = Some(report),
                Err(err) => warnings.push(format!("entropy scan failed: {}", err)),
            }
        }
        #[cfg(not(all(feature = "content", feature = "walk")))]
        warnings.push(
            crate::grid::DisabledFeature::EntropyProfiling
                .warning()
                .to_string(),
        );
    }
}

fn run_license(
    ctx: &AnalysisContext,
    req: &AnalysisRequest,
    plan: &PresetPlan,
    files: Option<&[PathBuf]>,
    outputs: &mut AnalysisOutputs,
    warnings: &mut Vec<String>,
) {
    if plan.license {
        #[cfg(all(feature = "content", feature = "walk"))]
        if let Some(list) = files {
            match build_license_report(&ctx.root, list, &req.limits) {
                Ok(report) => outputs.license = Some(report),
                Err(err) => warnings.push(format!("license scan failed: {}", err)),
            }
        }
        #[cfg(not(all(feature = "content", feature = "walk")))]
        warnings.push(
            crate::grid::DisabledFeature::LicenseRadar
                .warning()
                .to_string(),
        );
    }
}

fn run_complexity(
    ctx: &AnalysisContext,
    req: &AnalysisRequest,
    plan: &PresetPlan,
    files: Option<&[PathBuf]>,
    outputs: &mut AnalysisOutputs,
    warnings: &mut Vec<String>,
) {
    if plan.complexity {
        #[cfg(all(feature = "content", feature = "walk"))]
        if let Some(list) = files {
            match build_complexity_report(
                &ctx.root,
                list,
                &ctx.export,
                &req.limits,
                req.detail_functions,
            ) {
                Ok(report) => outputs.complexity = Some(report),
                Err(err) => warnings.push(format!("complexity scan failed: {}", err)),
            }
        }
        #[cfg(not(all(feature = "content", feature = "walk")))]
        warnings.push(
            crate::grid::DisabledFeature::ComplexityAnalysis
                .warning()
                .to_string(),
        );
    }
}

fn run_api_surface(
    ctx: &AnalysisContext,
    req: &AnalysisRequest,
    plan: &PresetPlan,
    files: Option<&[PathBuf]>,
    outputs: &mut AnalysisOutputs,
    warnings: &mut Vec<String>,
) {
    if plan.api_surface {
        #[cfg(all(feature = "content", feature = "walk"))]
        if let Some(list) = files {
            match build_api_surface_report(&ctx.root, list, &ctx.export, &req.limits) {
                Ok(report) => outputs.api_surface = Some(report),
                Err(err) => warnings.push(format!("api surface scan failed: {}", err)),
            }
        }
        #[cfg(not(all(feature = "content", feature = "walk")))]
        warnings.push(
            crate::grid::DisabledFeature::ApiSurfaceAnalysis
                .warning()
                .to_string(),
        );
    }
}

fn run_halstead(
    _ctx: &AnalysisContext,
    _req: &AnalysisRequest,
    _plan: &PresetPlan,
    _files: Option<&[PathBuf]>,
    _outputs: &mut AnalysisOutputs,
    _warnings: &mut Vec<String>,
) {
    #[cfg(all(feature = "halstead", feature = "content", feature = "walk"))]
    if _plan.halstead
        && let Some(list) = _files
    {
        match build_halstead_report(&_ctx.root, list, &_ctx.export, &_req.limits) {
            Ok(halstead_report) => {
                if let Some(ref mut complexity) = _outputs.complexity {
                    attach_halstead_metrics(complexity, halstead_report);
                }
            }
            Err(err) => _warnings.push(format!("halstead scan failed: {}", err)),
        }
    }
}

pub(super) fn run_git_enrichers(
    ctx: &AnalysisContext,
    req: &AnalysisRequest,
    plan: &PresetPlan,
    include_git: bool,
    has_host_root: bool,
    outputs: &mut AnalysisOutputs,
    warnings: &mut Vec<String>,
) {
    if !include_git {
        return;
    }

    #[cfg(feature = "git")]
    {
        if has_host_root {
            let repo_root = match tokmd_git::repo_root(&ctx.root) {
                Some(root) => root,
                None => {
                    warnings.push("git scan failed: not a git repo".to_string());
                    PathBuf::new()
                }
            };
            if !repo_root.as_os_str().is_empty() {
                collect_git_history(ctx, req, plan, &repo_root, outputs, warnings);
            }
        } else {
            push_warning_once(warnings, ROOTLESS_GIT_ANALYSIS_WARNING);
        }
    }
    #[cfg(not(feature = "git"))]
    warnings.push(
        crate::grid::DisabledFeature::GitMetrics
            .warning()
            .to_string(),
    );
}

#[cfg(feature = "git")]
fn collect_git_history(
    ctx: &AnalysisContext,
    req: &AnalysisRequest,
    plan: &PresetPlan,
    repo_root: &Path,
    outputs: &mut AnalysisOutputs,
    warnings: &mut Vec<String>,
) {
    match tokmd_git::collect_history(
        repo_root,
        req.limits.max_commits,
        req.limits.max_commit_files,
    ) {
        Ok(commits) => {
            if plan.git {
                match build_git_report(repo_root, &ctx.export, &commits) {
                    Ok(report) => outputs.git = Some(report),
                    Err(err) => warnings.push(format!("git scan failed: {}", err)),
                }
            }
            if plan.churn {
                outputs.churn = Some(build_predictive_churn_report(
                    &ctx.export,
                    &commits,
                    repo_root,
                ));
            }
            if plan.fingerprint {
                outputs.fingerprint = Some(build_corporate_fingerprint(&commits));
            }
        }
        Err(err) => warnings.push(format!("git scan failed: {}", err)),
    }
}

pub(super) fn run_metadata_enrichers(
    ctx: &AnalysisContext,
    plan: &PresetPlan,
    outputs: &mut AnalysisOutputs,
    warnings: &mut Vec<String>,
) {
    if plan.archetype {
        #[cfg(feature = "archetype")]
        {
            outputs.archetype = detect_archetype(&ctx.export);
        }
        #[cfg(not(feature = "archetype"))]
        warnings.push(
            crate::grid::DisabledFeature::Archetype
                .warning()
                .to_string(),
        );
    }

    if plan.topics {
        #[cfg(feature = "topics")]
        {
            outputs.topics = Some(build_topic_clouds(&ctx.export));
        }
        #[cfg(not(feature = "topics"))]
        warnings.push(crate::grid::DisabledFeature::Topics.warning().to_string());
    }
}

pub(super) fn run_fun_enricher(
    plan: &PresetPlan,
    derived: &DerivedReport,
    outputs: &mut AnalysisOutputs,
    warnings: &mut Vec<String>,
) {
    if plan.fun {
        #[cfg(feature = "fun")]
        {
            outputs.fun = Some(build_fun_report(derived));
        }
        #[cfg(not(feature = "fun"))]
        warnings.push(crate::grid::DisabledFeature::Fun.warning().to_string());
    }
}

pub(super) fn run_effort_enricher(
    ctx: &AnalysisContext,
    req: &AnalysisRequest,
    derived: &DerivedReport,
    outputs: &mut AnalysisOutputs,
    warnings: &mut Vec<String>,
) {
    #[cfg(feature = "effort")]
    if let Some(effort_request) = &req.effort {
        match build_effort_report(
            &ctx.root,
            &ctx.export,
            derived,
            outputs.git.as_ref(),
            outputs.complexity.as_ref(),
            outputs.api_surface.as_ref(),
            outputs.dup.as_ref(),
            effort_request,
        ) {
            Ok(report) => outputs.effort = Some(report),
            Err(err) => warnings.push(format!("effort estimate failed: {}", err)),
        }
    }
}
