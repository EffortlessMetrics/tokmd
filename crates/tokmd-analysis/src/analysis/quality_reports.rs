use std::path::{Path, PathBuf};

use tokmd_types::ExportData;

use crate::grid::PresetPlan;

use super::AnalysisRequest;
use super::reports::AnalysisReports;

pub(super) fn enrich_quality_reports(
    root: &Path,
    export: &ExportData,
    files: Option<&[PathBuf]>,
    plan: &PresetPlan,
    req: &AnalysisRequest,
    reports: &mut AnalysisReports,
    warnings: &mut Vec<String>,
) {
    enrich_entropy(root, export, files, plan, req, reports, warnings);
    enrich_license(root, files, plan, req, reports, warnings);
    enrich_complexity(root, export, files, plan, req, reports, warnings);
    enrich_api_surface(root, export, files, plan, req, reports, warnings);
    enrich_halstead(root, export, files, plan, req, reports, warnings);
}

fn enrich_entropy(
    root: &Path,
    export: &ExportData,
    files: Option<&[PathBuf]>,
    plan: &PresetPlan,
    req: &AnalysisRequest,
    reports: &mut AnalysisReports,
    warnings: &mut Vec<String>,
) {
    if !plan.entropy {
        return;
    }

    #[cfg(all(feature = "content", feature = "walk"))]
    if let Some(list) = files {
        match crate::entropy::build_entropy_report(root, list, export, &req.limits) {
            Ok(report) => reports.entropy = Some(report),
            Err(err) => warnings.push(format!("entropy scan failed: {}", err)),
        }
    }

    #[cfg(not(all(feature = "content", feature = "walk")))]
    warnings.push(
        crate::grid::DisabledFeature::EntropyProfiling
            .warning()
            .to_string(),
    );

    let _ = (root, export, files, req, reports, warnings);
}

fn enrich_license(
    root: &Path,
    files: Option<&[PathBuf]>,
    plan: &PresetPlan,
    req: &AnalysisRequest,
    reports: &mut AnalysisReports,
    warnings: &mut Vec<String>,
) {
    if !plan.license {
        return;
    }

    #[cfg(all(feature = "content", feature = "walk"))]
    if let Some(list) = files {
        match crate::license::build_license_report(root, list, &req.limits) {
            Ok(report) => reports.license = Some(report),
            Err(err) => warnings.push(format!("license scan failed: {}", err)),
        }
    }

    #[cfg(not(all(feature = "content", feature = "walk")))]
    warnings.push(
        crate::grid::DisabledFeature::LicenseRadar
            .warning()
            .to_string(),
    );

    let _ = (root, files, req, reports, warnings);
}

fn enrich_complexity(
    root: &Path,
    export: &ExportData,
    files: Option<&[PathBuf]>,
    plan: &PresetPlan,
    req: &AnalysisRequest,
    reports: &mut AnalysisReports,
    warnings: &mut Vec<String>,
) {
    if !plan.complexity {
        return;
    }

    #[cfg(all(feature = "content", feature = "walk"))]
    if let Some(list) = files {
        match crate::complexity::build_complexity_report(
            root,
            list,
            export,
            &req.limits,
            req.detail_functions,
        ) {
            Ok(report) => reports.complexity = Some(report),
            Err(err) => warnings.push(format!("complexity scan failed: {}", err)),
        }
    }

    #[cfg(not(all(feature = "content", feature = "walk")))]
    warnings.push(
        crate::grid::DisabledFeature::ComplexityAnalysis
            .warning()
            .to_string(),
    );

    let _ = (root, export, files, req, reports, warnings);
}

fn enrich_api_surface(
    root: &Path,
    export: &ExportData,
    files: Option<&[PathBuf]>,
    plan: &PresetPlan,
    req: &AnalysisRequest,
    reports: &mut AnalysisReports,
    warnings: &mut Vec<String>,
) {
    if !plan.api_surface {
        return;
    }

    #[cfg(all(feature = "content", feature = "walk"))]
    if let Some(list) = files {
        match crate::api_surface::build_api_surface_report(root, list, export, &req.limits) {
            Ok(report) => reports.api_surface = Some(report),
            Err(err) => warnings.push(format!("api surface scan failed: {}", err)),
        }
    }

    #[cfg(not(all(feature = "content", feature = "walk")))]
    warnings.push(
        crate::grid::DisabledFeature::ApiSurfaceAnalysis
            .warning()
            .to_string(),
    );

    let _ = (root, export, files, req, reports, warnings);
}

fn enrich_halstead(
    root: &Path,
    export: &ExportData,
    files: Option<&[PathBuf]>,
    plan: &PresetPlan,
    req: &AnalysisRequest,
    reports: &mut AnalysisReports,
    warnings: &mut Vec<String>,
) {
    #[cfg(all(feature = "halstead", feature = "content", feature = "walk"))]
    if plan.halstead
        && let Some(list) = files
    {
        match crate::halstead::build_halstead_report(root, list, export, &req.limits) {
            Ok(halstead_report) => {
                if let Some(ref mut complexity) = reports.complexity {
                    crate::maintainability::attach_halstead_metrics(complexity, halstead_report);
                }
            }
            Err(err) => warnings.push(format!("halstead scan failed: {}", err)),
        }
    }

    let _ = (root, export, files, plan, req, reports, warnings);
}
