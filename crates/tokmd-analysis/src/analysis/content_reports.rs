use std::path::{Path, PathBuf};

use tokmd_analysis_types::DerivedReport;
use tokmd_types::ExportData;

use crate::grid::PresetPlan;

use super::AnalysisRequest;
use super::reports::AnalysisReports;
#[cfg(feature = "content")]
use super::{ROOTLESS_FILE_ANALYSIS_WARNING, has_host_root, push_warning_once};

pub(super) struct ContentEnrichmentContext<'a> {
    pub(super) root: &'a Path,
    pub(super) export: &'a ExportData,
    pub(super) files: Option<&'a [PathBuf]>,
    pub(super) plan: &'a PresetPlan,
    pub(super) req: &'a AnalysisRequest,
}

pub(super) fn enrich_content_reports(
    ctx: ContentEnrichmentContext<'_>,
    derived: &mut DerivedReport,
    reports: &mut AnalysisReports,
    warnings: &mut Vec<String>,
) {
    #[cfg(not(feature = "content"))]
    let _ = (ctx.root, ctx.export, ctx.files);

    enrich_todo(&ctx, derived, warnings);
    enrich_duplicates(&ctx, reports, warnings);
    enrich_near_duplicates(&ctx, reports, warnings);
    enrich_imports(&ctx, reports, warnings);
}

fn enrich_todo(
    ctx: &ContentEnrichmentContext<'_>,
    derived: &mut DerivedReport,
    warnings: &mut Vec<String>,
) {
    if !ctx.plan.todo {
        return;
    }

    #[cfg(feature = "content")]
    if let Some(list) = ctx.files {
        let limits = super::content_limits(&ctx.req.limits);
        match crate::content::build_todo_report(ctx.root, list, &limits, derived.totals.code) {
            Ok(report) => derived.todo = Some(report),
            Err(err) => warnings.push(format!("todo scan failed: {}", err)),
        }
    }

    #[cfg(not(feature = "content"))]
    warnings.push(crate::grid::DisabledFeature::TodoScan.warning().to_string());

    let _ = (ctx, derived, warnings);
}

fn enrich_duplicates(
    ctx: &ContentEnrichmentContext<'_>,
    reports: &mut AnalysisReports,
    warnings: &mut Vec<String>,
) {
    if !ctx.plan.dup {
        return;
    }

    #[cfg(feature = "content")]
    if let Some(list) = ctx.files {
        let limits = super::content_limits(&ctx.req.limits);
        match crate::content::build_duplicate_report(ctx.root, list, ctx.export, &limits) {
            Ok(report) => reports.dup = Some(report),
            Err(err) => warnings.push(format!("dup scan failed: {}", err)),
        }
    }

    #[cfg(not(feature = "content"))]
    warnings.push(
        crate::grid::DisabledFeature::DuplicationScan
            .warning()
            .to_string(),
    );

    let _ = (ctx, reports, warnings);
}

fn enrich_near_duplicates(
    ctx: &ContentEnrichmentContext<'_>,
    reports: &mut AnalysisReports,
    warnings: &mut Vec<String>,
) {
    if !ctx.req.near_dup {
        return;
    }

    #[cfg(feature = "content")]
    {
        if has_host_root(ctx.root) {
            let near_dup_limits = crate::near_dup::NearDupLimits {
                max_bytes: ctx.req.limits.max_bytes,
                max_file_bytes: ctx.req.limits.max_file_bytes,
            };
            match crate::near_dup::build_near_dup_report(
                ctx.root,
                ctx.export,
                ctx.req.near_dup_scope,
                ctx.req.near_dup_threshold,
                ctx.req.near_dup_max_files,
                ctx.req.near_dup_max_pairs,
                &near_dup_limits,
                &ctx.req.near_dup_exclude,
            ) {
                Ok(report) => attach_near_duplicate_report(reports, report),
                Err(err) => warnings.push(format!("near-dup scan failed: {}", err)),
            }
        } else {
            push_warning_once(warnings, ROOTLESS_FILE_ANALYSIS_WARNING);
        }
    }

    #[cfg(not(feature = "content"))]
    warnings.push(
        crate::grid::DisabledFeature::NearDuplicateScan
            .warning()
            .to_string(),
    );

    let _ = (ctx, reports, warnings);
}

#[cfg(feature = "content")]
fn attach_near_duplicate_report(
    reports: &mut AnalysisReports,
    report: tokmd_analysis_types::NearDuplicateReport,
) {
    if let Some(ref mut dup) = reports.dup {
        dup.near = Some(report);
    } else {
        reports.dup = Some(tokmd_analysis_types::DuplicateReport {
            groups: Vec::new(),
            wasted_bytes: 0,
            strategy: "none".to_string(),
            density: None,
            near: Some(report),
        });
    }
}

fn enrich_imports(
    ctx: &ContentEnrichmentContext<'_>,
    reports: &mut AnalysisReports,
    warnings: &mut Vec<String>,
) {
    if !ctx.plan.imports {
        return;
    }

    #[cfg(feature = "content")]
    if let Some(list) = ctx.files {
        let limits = super::content_limits(&ctx.req.limits);
        let granularity = super::content_import_granularity(ctx.req.import_granularity);
        match crate::content::build_import_report(ctx.root, list, ctx.export, granularity, &limits)
        {
            Ok(report) => reports.imports = Some(report),
            Err(err) => warnings.push(format!("import scan failed: {}", err)),
        }
    }

    #[cfg(not(feature = "content"))]
    warnings.push(
        crate::grid::DisabledFeature::ImportScan
            .warning()
            .to_string(),
    );

    let _ = (ctx, reports, warnings);
}
