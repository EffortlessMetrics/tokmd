#[cfg(feature = "effort")]
use std::path::Path;

use tokmd_analysis_types::DerivedReport;
#[cfg(feature = "effort")]
use tokmd_types::ExportData;

use crate::grid::PresetPlan;

#[cfg(feature = "effort")]
use crate::effort::{EffortRequest, build_effort_report};

use super::reports::AnalysisReports;

pub(super) fn enrich_fun_report(
    plan: &PresetPlan,
    derived: &DerivedReport,
    reports: &mut AnalysisReports,
    warnings: &mut Vec<String>,
) {
    if plan.fun {
        #[cfg(feature = "fun")]
        {
            reports.fun = Some(crate::fun::build_fun_report(derived));
        }
        #[cfg(not(feature = "fun"))]
        {
            warnings.push(crate::grid::DisabledFeature::Fun.warning().to_string());
            reports.fun = None;
        }
    } else {
        reports.fun = None;
    }

    let _ = (plan, derived, reports, warnings);
}

#[cfg(feature = "effort")]
pub(super) fn enrich_effort_report(
    root: &Path,
    export: &ExportData,
    derived: &DerivedReport,
    effort: Option<&EffortRequest>,
    reports: &mut AnalysisReports,
    warnings: &mut Vec<String>,
) {
    let Some(effort_request) = effort else {
        return;
    };

    match build_effort_report(
        root,
        export,
        derived,
        reports.git.as_ref(),
        reports.complexity.as_ref(),
        reports.api_surface.as_ref(),
        reports.dup.as_ref(),
        effort_request,
    ) {
        Ok(report) => reports.effort = Some(report),
        Err(err) => warnings.push(format!("effort estimate failed: {}", err)),
    }
}
