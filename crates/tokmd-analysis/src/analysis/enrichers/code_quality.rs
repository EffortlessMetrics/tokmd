#![cfg_attr(
    not(all(feature = "content", feature = "walk")),
    allow(unused_variables)
)]
use std::path::{Path, PathBuf};

use tokmd_analysis_types::AnalysisLimits;
use tokmd_types::ExportData;

use crate::grid::PresetPlan;

use super::super::outputs::AnalysisOutputs;

#[cfg_attr(not(all(feature = "content", feature = "walk")), allow(dead_code))]
pub(in crate::analysis) struct CodeQualityInput<'a> {
    pub(in crate::analysis) root: &'a Path,
    pub(in crate::analysis) export: &'a ExportData,
    pub(in crate::analysis) files: Option<&'a [PathBuf]>,
    pub(in crate::analysis) plan: &'a PresetPlan,
    pub(in crate::analysis) limits: &'a AnalysisLimits,
    pub(in crate::analysis) detail_functions: bool,
}

pub(in crate::analysis) fn run(
    input: CodeQualityInput<'_>,
    outputs: &mut AnalysisOutputs,
    warnings: &mut Vec<String>,
) {
    run_entropy(&input, outputs, warnings);
    run_license(&input, outputs, warnings);
    run_complexity(&input, outputs, warnings);
    run_api_surface(&input, outputs, warnings);
    attach_halstead(&input, outputs, warnings);
}

fn run_entropy(
    input: &CodeQualityInput<'_>,
    outputs: &mut AnalysisOutputs,
    warnings: &mut Vec<String>,
) {
    if input.plan.entropy {
        #[cfg(all(feature = "content", feature = "walk"))]
        if let Some(list) = input.files {
            match crate::entropy::build_entropy_report(input.root, list, input.export, input.limits)
            {
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
    input: &CodeQualityInput<'_>,
    outputs: &mut AnalysisOutputs,
    warnings: &mut Vec<String>,
) {
    if input.plan.license {
        #[cfg(all(feature = "content", feature = "walk"))]
        if let Some(list) = input.files {
            match crate::license::build_license_report(input.root, list, input.limits) {
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
    input: &CodeQualityInput<'_>,
    outputs: &mut AnalysisOutputs,
    warnings: &mut Vec<String>,
) {
    if input.plan.complexity {
        #[cfg(all(feature = "content", feature = "walk"))]
        if let Some(list) = input.files {
            match crate::complexity::build_complexity_report(
                input.root,
                list,
                input.export,
                input.limits,
                input.detail_functions,
            ) {
                Ok(report) => {
                    outputs.complexity = Some(report);
                    for warning in crate::complexity::bounded_complexity_warnings(
                        input.root,
                        list,
                        input.export,
                        input.limits,
                    ) {
                        if warnings.iter().all(|existing| existing != &warning) {
                            warnings.push(warning);
                        }
                    }
                }
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
    input: &CodeQualityInput<'_>,
    outputs: &mut AnalysisOutputs,
    warnings: &mut Vec<String>,
) {
    if input.plan.api_surface {
        #[cfg(all(feature = "content", feature = "walk"))]
        if let Some(list) = input.files {
            match crate::api_surface::build_api_surface_report(
                input.root,
                list,
                input.export,
                input.limits,
            ) {
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

fn attach_halstead(
    input: &CodeQualityInput<'_>,
    outputs: &mut AnalysisOutputs,
    warnings: &mut Vec<String>,
) {
    #[cfg(all(feature = "halstead", feature = "content", feature = "walk"))]
    if input.plan.halstead
        && let Some(list) = input.files
    {
        match crate::halstead::build_halstead_report(input.root, list, input.export, input.limits) {
            Ok(halstead_report) => {
                if let Some(ref mut complexity) = outputs.complexity {
                    crate::maintainability::attach_halstead_metrics(complexity, halstead_report);
                }
            }
            Err(err) => warnings.push(format!("halstead scan failed: {}", err)),
        }
    }
    #[cfg(not(all(feature = "halstead", feature = "content", feature = "walk")))]
    let _ = (input, outputs, warnings);
}
