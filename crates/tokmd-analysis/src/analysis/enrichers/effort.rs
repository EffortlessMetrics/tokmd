use std::path::Path;

use tokmd_analysis_types::{DerivedReport, EffortEstimateReport};
use tokmd_types::ExportData;

use crate::effort::{EffortRequest, build_effort_report};

use super::super::outputs::AnalysisOutputs;

pub(in crate::analysis) fn run(
    root: &Path,
    export: &ExportData,
    derived: &DerivedReport,
    outputs: &AnalysisOutputs,
    request: Option<&EffortRequest>,
    warnings: &mut Vec<String>,
) -> Option<EffortEstimateReport> {
    let effort_request = request?;
    match build_effort_report(
        root,
        export,
        derived,
        outputs.git.as_ref(),
        outputs.complexity.as_ref(),
        outputs.api_surface.as_ref(),
        outputs.dup.as_ref(),
        effort_request,
    ) {
        Ok(report) => Some(report),
        Err(err) => {
            warnings.push(format!("effort estimate failed: {}", err));
            None
        }
    }
}
