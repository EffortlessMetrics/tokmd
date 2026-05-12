use tokmd_types::ExportData;

use crate::grid::PresetPlan;

use super::reports::AnalysisReports;

pub(super) fn enrich_identity_reports(
    export: &ExportData,
    plan: &PresetPlan,
    reports: &mut AnalysisReports,
    warnings: &mut Vec<String>,
) {
    if plan.archetype {
        #[cfg(feature = "archetype")]
        {
            reports.archetype = crate::archetype::detect_archetype(export);
        }
        #[cfg(not(feature = "archetype"))]
        {
            warnings.push(
                crate::grid::DisabledFeature::Archetype
                    .warning()
                    .to_string(),
            );
        }
    }

    if plan.topics {
        #[cfg(feature = "topics")]
        {
            reports.topics = Some(crate::topics::build_topic_clouds(export));
        }
        #[cfg(not(feature = "topics"))]
        {
            warnings.push(crate::grid::DisabledFeature::Topics.warning().to_string());
        }
    }

    let _ = (export, plan, reports, warnings);
}
