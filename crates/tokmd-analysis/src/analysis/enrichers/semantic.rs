use tokmd_analysis_types::DerivedReport;
use tokmd_types::ExportData;

use crate::grid::PresetPlan;

use super::super::outputs::AnalysisOutputs;

pub(in crate::analysis) fn run(
    export: &ExportData,
    derived: &DerivedReport,
    plan: &PresetPlan,
    outputs: &mut AnalysisOutputs,
    warnings: &mut Vec<String>,
) {
    run_archetype(export, plan, outputs, warnings);
    run_topics(export, plan, outputs, warnings);
    run_fun(derived, plan, outputs, warnings);
}

fn run_archetype(
    export: &ExportData,
    plan: &PresetPlan,
    outputs: &mut AnalysisOutputs,
    warnings: &mut Vec<String>,
) {
    let _ = warnings;
    if plan.archetype {
        #[cfg(feature = "archetype")]
        {
            outputs.archetype = crate::archetype::detect_archetype(export);
        }
        #[cfg(not(feature = "archetype"))]
        {
            let _ = (export, outputs);
            warnings.push(
                crate::grid::DisabledFeature::Archetype
                    .warning()
                    .to_string(),
            );
        }
    }
}

fn run_topics(
    export: &ExportData,
    plan: &PresetPlan,
    outputs: &mut AnalysisOutputs,
    warnings: &mut Vec<String>,
) {
    let _ = warnings;
    if plan.topics {
        #[cfg(feature = "topics")]
        {
            outputs.topics = Some(crate::topics::build_topic_clouds(export));
        }
        #[cfg(not(feature = "topics"))]
        {
            let _ = (export, outputs);
            warnings.push(crate::grid::DisabledFeature::Topics.warning().to_string());
        }
    }
}

fn run_fun(
    derived: &DerivedReport,
    plan: &PresetPlan,
    outputs: &mut AnalysisOutputs,
    warnings: &mut Vec<String>,
) {
    let _ = warnings;
    if plan.fun {
        #[cfg(feature = "fun")]
        {
            outputs.fun = Some(crate::fun::build_fun_report(derived));
        }
        #[cfg(not(feature = "fun"))]
        {
            let _ = (derived, outputs);
            warnings.push(crate::grid::DisabledFeature::Fun.warning().to_string());
        }
    }
}
