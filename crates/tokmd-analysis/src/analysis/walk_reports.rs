use std::path::{Path, PathBuf};

use crate::grid::PresetPlan;

use super::reports::AnalysisReports;

pub(super) fn enrich_walk_reports(
    root: &Path,
    files: Option<&[PathBuf]>,
    plan: &PresetPlan,
    reports: &mut AnalysisReports,
    warnings: &mut Vec<String>,
) {
    if plan.assets {
        #[cfg(feature = "walk")]
        if let Some(list) = files {
            match crate::assets::build_assets_report(root, list) {
                Ok(report) => reports.assets = Some(report),
                Err(err) => warnings.push(format!("asset scan failed: {}", err)),
            }
        }
    }

    if plan.deps {
        #[cfg(feature = "walk")]
        if let Some(list) = files {
            match crate::assets::build_dependency_report(root, list) {
                Ok(report) => reports.deps = Some(report),
                Err(err) => warnings.push(format!("dependency scan failed: {}", err)),
            }
        }
    }

    let _ = (root, files, plan, reports, warnings);
}
