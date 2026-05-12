use tokmd_analysis_types::DerivedReport;
#[cfg(feature = "content")]
use tokmd_analysis_types::DuplicateReport;

#[cfg(feature = "archetype")]
use crate::archetype::detect_archetype;
#[cfg(feature = "walk")]
use crate::assets::{build_assets_report, build_dependency_report};
#[cfg(feature = "content")]
use crate::content::{
    ContentLimits, ImportGranularity as ContentImportGranularity, build_duplicate_report,
    build_import_report, build_todo_report,
};
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
    api_surface::build_api_surface_report, complexity::build_complexity_report,
    entropy::build_entropy_report, license::build_license_report,
};
#[cfg(all(feature = "halstead", feature = "content", feature = "walk"))]
use crate::{halstead::build_halstead_report, maintainability::attach_halstead_metrics};

#[cfg(feature = "content")]
use super::ImportGranularity;
use super::report_set::AnalysisReports;
#[cfg(any(feature = "content", feature = "git"))]
use super::warnings;
use super::{AnalysisContext, AnalysisRequest};

#[cfg(feature = "content")]
fn content_limits(limits: &tokmd_analysis_types::AnalysisLimits) -> ContentLimits {
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

pub(super) fn run_file_enrichers(
    ctx: &AnalysisContext,
    req: &AnalysisRequest,
    plan: &PresetPlan,
    files: Option<&[std::path::PathBuf]>,
    derived: &mut DerivedReport,
    reports: &mut AnalysisReports,
    warnings_out: &mut Vec<String>,
) {
    run_asset_enrichers(ctx, plan, files, reports, warnings_out);
    run_content_enrichers(ctx, req, plan, files, derived, reports, warnings_out);
    run_content_walk_enrichers(ctx, req, plan, files, reports, warnings_out);
}

fn run_asset_enrichers(
    ctx: &AnalysisContext,
    plan: &PresetPlan,
    files: Option<&[std::path::PathBuf]>,
    reports: &mut AnalysisReports,
    warnings_out: &mut Vec<String>,
) {
    if plan.assets {
        #[cfg(feature = "walk")]
        if let Some(list) = files {
            match build_assets_report(&ctx.root, list) {
                Ok(report) => reports.assets = Some(report),
                Err(err) => warnings_out.push(format!("asset scan failed: {}", err)),
            }
        }
    }

    if plan.deps {
        #[cfg(feature = "walk")]
        if let Some(list) = files {
            match build_dependency_report(&ctx.root, list) {
                Ok(report) => reports.deps = Some(report),
                Err(err) => warnings_out.push(format!("dependency scan failed: {}", err)),
            }
        }
    }

    #[cfg(not(feature = "walk"))]
    let _ = (ctx, files, reports, warnings_out);
}

fn run_content_enrichers(
    ctx: &AnalysisContext,
    req: &AnalysisRequest,
    plan: &PresetPlan,
    files: Option<&[std::path::PathBuf]>,
    derived: &mut DerivedReport,
    reports: &mut AnalysisReports,
    warnings_out: &mut Vec<String>,
) {
    run_todo_enricher(ctx, req, plan, files, derived, warnings_out);
    run_duplicate_enrichers(ctx, req, plan, files, reports, warnings_out);
    run_import_enricher(ctx, req, plan, files, reports, warnings_out);
}

fn run_todo_enricher(
    ctx: &AnalysisContext,
    req: &AnalysisRequest,
    plan: &PresetPlan,
    files: Option<&[std::path::PathBuf]>,
    derived: &mut DerivedReport,
    warnings_out: &mut Vec<String>,
) {
    if !plan.todo {
        return;
    }

    #[cfg(feature = "content")]
    if let Some(list) = files {
        let limits = content_limits(&req.limits);
        match build_todo_report(&ctx.root, list, &limits, derived.totals.code) {
            Ok(report) => derived.todo = Some(report),
            Err(err) => warnings_out.push(format!("todo scan failed: {}", err)),
        }
    }

    #[cfg(not(feature = "content"))]
    {
        let _ = (ctx, req, files, derived);
        warnings_out.push(crate::grid::DisabledFeature::TodoScan.warning().to_string());
    }
}

fn run_duplicate_enrichers(
    ctx: &AnalysisContext,
    req: &AnalysisRequest,
    plan: &PresetPlan,
    files: Option<&[std::path::PathBuf]>,
    reports: &mut AnalysisReports,
    warnings_out: &mut Vec<String>,
) {
    #[cfg(not(feature = "content"))]
    let _ = (ctx, files, reports);

    if plan.dup {
        #[cfg(feature = "content")]
        if let Some(list) = files {
            let limits = content_limits(&req.limits);
            match build_duplicate_report(&ctx.root, list, &ctx.export, &limits) {
                Ok(report) => reports.dup = Some(report),
                Err(err) => warnings_out.push(format!("dup scan failed: {}", err)),
            }
        }

        #[cfg(not(feature = "content"))]
        warnings_out.push(
            crate::grid::DisabledFeature::DuplicationScan
                .warning()
                .to_string(),
        );
    }

    if req.near_dup {
        #[cfg(feature = "content")]
        {
            if warnings::has_host_root(&ctx.root) {
                let limits = NearDupLimits {
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
                    &limits,
                    &req.near_dup_exclude,
                ) {
                    Ok(report) => attach_near_duplicate_report(reports, report),
                    Err(err) => warnings_out.push(format!("near-dup scan failed: {}", err)),
                }
            } else {
                warnings::push_warning_once(warnings_out, warnings::ROOTLESS_FILE_ANALYSIS_WARNING);
            }
        }

        #[cfg(not(feature = "content"))]
        warnings_out.push(
            crate::grid::DisabledFeature::NearDuplicateScan
                .warning()
                .to_string(),
        );
    }
}

#[cfg(feature = "content")]
fn attach_near_duplicate_report(
    reports: &mut AnalysisReports,
    report: tokmd_analysis_types::NearDuplicateReport,
) {
    if let Some(ref mut dup) = reports.dup {
        dup.near = Some(report);
    } else {
        reports.dup = Some(DuplicateReport {
            groups: Vec::new(),
            wasted_bytes: 0,
            strategy: "none".to_string(),
            density: None,
            near: Some(report),
        });
    }
}

fn run_import_enricher(
    ctx: &AnalysisContext,
    req: &AnalysisRequest,
    plan: &PresetPlan,
    files: Option<&[std::path::PathBuf]>,
    reports: &mut AnalysisReports,
    warnings_out: &mut Vec<String>,
) {
    if !plan.imports {
        return;
    }

    #[cfg(feature = "content")]
    if let Some(list) = files {
        let limits = content_limits(&req.limits);
        let granularity = content_import_granularity(req.import_granularity);
        match build_import_report(&ctx.root, list, &ctx.export, granularity, &limits) {
            Ok(report) => reports.imports = Some(report),
            Err(err) => warnings_out.push(format!("import scan failed: {}", err)),
        }
    }

    #[cfg(not(feature = "content"))]
    {
        let _ = (ctx, req, files, reports);
        warnings_out.push(
            crate::grid::DisabledFeature::ImportScan
                .warning()
                .to_string(),
        );
    }
}

fn run_content_walk_enrichers(
    ctx: &AnalysisContext,
    req: &AnalysisRequest,
    plan: &PresetPlan,
    files: Option<&[std::path::PathBuf]>,
    reports: &mut AnalysisReports,
    warnings_out: &mut Vec<String>,
) {
    run_entropy_enricher(ctx, req, plan, files, reports, warnings_out);
    run_license_enricher(ctx, req, plan, files, reports, warnings_out);
    run_complexity_enricher(ctx, req, plan, files, reports, warnings_out);
    run_api_surface_enricher(ctx, req, plan, files, reports, warnings_out);
    run_halstead_enricher(ctx, req, plan, files, reports, warnings_out);
}

fn run_entropy_enricher(
    ctx: &AnalysisContext,
    req: &AnalysisRequest,
    plan: &PresetPlan,
    files: Option<&[std::path::PathBuf]>,
    reports: &mut AnalysisReports,
    warnings_out: &mut Vec<String>,
) {
    if !plan.entropy {
        return;
    }

    #[cfg(all(feature = "content", feature = "walk"))]
    if let Some(list) = files {
        match build_entropy_report(&ctx.root, list, &ctx.export, &req.limits) {
            Ok(report) => reports.entropy = Some(report),
            Err(err) => warnings_out.push(format!("entropy scan failed: {}", err)),
        }
    }

    #[cfg(not(all(feature = "content", feature = "walk")))]
    {
        let _ = (ctx, req, files, reports);
        warnings_out.push(
            crate::grid::DisabledFeature::EntropyProfiling
                .warning()
                .to_string(),
        );
    }
}

fn run_license_enricher(
    ctx: &AnalysisContext,
    req: &AnalysisRequest,
    plan: &PresetPlan,
    files: Option<&[std::path::PathBuf]>,
    reports: &mut AnalysisReports,
    warnings_out: &mut Vec<String>,
) {
    if !plan.license {
        return;
    }

    #[cfg(all(feature = "content", feature = "walk"))]
    if let Some(list) = files {
        match build_license_report(&ctx.root, list, &req.limits) {
            Ok(report) => reports.license = Some(report),
            Err(err) => warnings_out.push(format!("license scan failed: {}", err)),
        }
    }

    #[cfg(not(all(feature = "content", feature = "walk")))]
    {
        let _ = (ctx, req, files, reports);
        warnings_out.push(
            crate::grid::DisabledFeature::LicenseRadar
                .warning()
                .to_string(),
        );
    }
}

fn run_complexity_enricher(
    ctx: &AnalysisContext,
    req: &AnalysisRequest,
    plan: &PresetPlan,
    files: Option<&[std::path::PathBuf]>,
    reports: &mut AnalysisReports,
    warnings_out: &mut Vec<String>,
) {
    if !plan.complexity {
        return;
    }

    #[cfg(all(feature = "content", feature = "walk"))]
    if let Some(list) = files {
        match build_complexity_report(
            &ctx.root,
            list,
            &ctx.export,
            &req.limits,
            req.detail_functions,
        ) {
            Ok(report) => reports.complexity = Some(report),
            Err(err) => warnings_out.push(format!("complexity scan failed: {}", err)),
        }
    }

    #[cfg(not(all(feature = "content", feature = "walk")))]
    {
        let _ = (ctx, req, files, reports);
        warnings_out.push(
            crate::grid::DisabledFeature::ComplexityAnalysis
                .warning()
                .to_string(),
        );
    }
}

fn run_api_surface_enricher(
    ctx: &AnalysisContext,
    req: &AnalysisRequest,
    plan: &PresetPlan,
    files: Option<&[std::path::PathBuf]>,
    reports: &mut AnalysisReports,
    warnings_out: &mut Vec<String>,
) {
    if !plan.api_surface {
        return;
    }

    #[cfg(all(feature = "content", feature = "walk"))]
    if let Some(list) = files {
        match build_api_surface_report(&ctx.root, list, &ctx.export, &req.limits) {
            Ok(report) => reports.api_surface = Some(report),
            Err(err) => warnings_out.push(format!("api surface scan failed: {}", err)),
        }
    }

    #[cfg(not(all(feature = "content", feature = "walk")))]
    {
        let _ = (ctx, req, files, reports);
        warnings_out.push(
            crate::grid::DisabledFeature::ApiSurfaceAnalysis
                .warning()
                .to_string(),
        );
    }
}

fn run_halstead_enricher(
    ctx: &AnalysisContext,
    req: &AnalysisRequest,
    plan: &PresetPlan,
    files: Option<&[std::path::PathBuf]>,
    reports: &mut AnalysisReports,
    warnings_out: &mut Vec<String>,
) {
    #[cfg(all(feature = "halstead", feature = "content", feature = "walk"))]
    if plan.halstead
        && let Some(list) = files
    {
        match build_halstead_report(&ctx.root, list, &ctx.export, &req.limits) {
            Ok(halstead_report) => {
                if let Some(ref mut complexity) = reports.complexity {
                    attach_halstead_metrics(complexity, halstead_report);
                }
            }
            Err(err) => warnings_out.push(format!("halstead scan failed: {}", err)),
        }
    }

    #[cfg(not(all(feature = "halstead", feature = "content", feature = "walk")))]
    let _ = (ctx, req, plan, files, reports, warnings_out);
}

pub(super) fn run_git_enrichers(
    ctx: &AnalysisContext,
    req: &AnalysisRequest,
    plan: &PresetPlan,
    include_git: bool,
    reports: &mut AnalysisReports,
    warnings_out: &mut Vec<String>,
) {
    if !include_git {
        return;
    }

    #[cfg(feature = "git")]
    {
        if warnings::has_host_root(&ctx.root) {
            collect_git_reports(ctx, req, plan, reports, warnings_out);
        } else {
            warnings::push_warning_once(warnings_out, warnings::ROOTLESS_GIT_ANALYSIS_WARNING);
        }
    }

    #[cfg(not(feature = "git"))]
    {
        let _ = (ctx, req, plan, reports);
        warnings_out.push(
            crate::grid::DisabledFeature::GitMetrics
                .warning()
                .to_string(),
        );
    }
}

#[cfg(feature = "git")]
fn collect_git_reports(
    ctx: &AnalysisContext,
    req: &AnalysisRequest,
    plan: &PresetPlan,
    reports: &mut AnalysisReports,
    warnings_out: &mut Vec<String>,
) {
    let Some(repo_root) = tokmd_git::repo_root(&ctx.root) else {
        warnings_out.push("git scan failed: not a git repo".to_string());
        return;
    };

    match tokmd_git::collect_history(
        &repo_root,
        req.limits.max_commits,
        req.limits.max_commit_files,
    ) {
        Ok(commits) => {
            if plan.git {
                match build_git_report(&repo_root, &ctx.export, &commits) {
                    Ok(report) => reports.git = Some(report),
                    Err(err) => warnings_out.push(format!("git scan failed: {}", err)),
                }
            }
            if plan.churn {
                reports.predictive_churn = Some(build_predictive_churn_report(
                    &ctx.export,
                    &commits,
                    &repo_root,
                ));
            }
            if plan.fingerprint {
                reports.corporate_fingerprint = Some(build_corporate_fingerprint(&commits));
            }
        }
        Err(err) => warnings_out.push(format!("git scan failed: {}", err)),
    }
}

pub(super) fn run_model_enrichers(
    ctx: &AnalysisContext,
    plan: &PresetPlan,
    derived: &DerivedReport,
    reports: &mut AnalysisReports,
    warnings_out: &mut Vec<String>,
) {
    run_archetype_enricher(ctx, plan, reports, warnings_out);
    run_topics_enricher(ctx, plan, reports, warnings_out);
    run_fun_enricher(plan, derived, reports, warnings_out);
}

fn run_archetype_enricher(
    ctx: &AnalysisContext,
    plan: &PresetPlan,
    reports: &mut AnalysisReports,
    warnings_out: &mut Vec<String>,
) {
    if !plan.archetype {
        return;
    }

    #[cfg(feature = "archetype")]
    {
        let _ = warnings_out;
        reports.archetype = detect_archetype(&ctx.export);
    }

    #[cfg(not(feature = "archetype"))]
    {
        let _ = (ctx, reports);
        warnings_out.push(
            crate::grid::DisabledFeature::Archetype
                .warning()
                .to_string(),
        );
    }
}

fn run_topics_enricher(
    ctx: &AnalysisContext,
    plan: &PresetPlan,
    reports: &mut AnalysisReports,
    warnings_out: &mut Vec<String>,
) {
    if !plan.topics {
        return;
    }

    #[cfg(feature = "topics")]
    {
        let _ = warnings_out;
        reports.topics = Some(build_topic_clouds(&ctx.export));
    }

    #[cfg(not(feature = "topics"))]
    {
        let _ = (ctx, reports);
        warnings_out.push(crate::grid::DisabledFeature::Topics.warning().to_string());
    }
}

fn run_fun_enricher(
    plan: &PresetPlan,
    derived: &DerivedReport,
    reports: &mut AnalysisReports,
    warnings_out: &mut Vec<String>,
) {
    if !plan.fun {
        reports.fun = None;
        return;
    }

    #[cfg(feature = "fun")]
    {
        let _ = warnings_out;
        reports.fun = Some(build_fun_report(derived));
    }

    #[cfg(not(feature = "fun"))]
    {
        let _ = derived;
        warnings_out.push(crate::grid::DisabledFeature::Fun.warning().to_string());
        reports.fun = None;
    }
}

pub(super) fn run_effort_enricher(
    ctx: &AnalysisContext,
    req: &AnalysisRequest,
    derived: &DerivedReport,
    reports: &mut AnalysisReports,
    warnings_out: &mut Vec<String>,
) {
    #[cfg(feature = "effort")]
    if let Some(effort_request) = &req.effort {
        match build_effort_report(
            &ctx.root,
            &ctx.export,
            derived,
            reports.git.as_ref(),
            reports.complexity.as_ref(),
            reports.api_surface.as_ref(),
            reports.dup.as_ref(),
            effort_request,
        ) {
            Ok(report) => reports.effort = Some(report),
            Err(err) => warnings_out.push(format!("effort estimate failed: {}", err)),
        }
    }

    #[cfg(not(feature = "effort"))]
    let _ = (ctx, req, derived, reports, warnings_out);
}
