#![cfg_attr(not(feature = "content"), allow(unused_imports, unused_variables))]
use std::path::{Path, PathBuf};

use tokmd_analysis_types::{AnalysisLimits, DerivedReport, DuplicateReport};
use tokmd_types::ExportData;

use crate::grid::PresetPlan;

#[cfg(feature = "content")]
use super::super::files::{ROOTLESS_FILE_ANALYSIS_WARNING, push_warning_once};
use super::super::outputs::AnalysisOutputs;
use super::super::{AnalysisRequest, ImportGranularity};

#[cfg_attr(not(feature = "content"), allow(dead_code))]
pub(in crate::analysis) struct ContentInput<'a> {
    pub(in crate::analysis) root: &'a Path,
    pub(in crate::analysis) export: &'a ExportData,
    pub(in crate::analysis) files: Option<&'a [PathBuf]>,
    pub(in crate::analysis) plan: &'a PresetPlan,
    pub(in crate::analysis) req: &'a AnalysisRequest,
    pub(in crate::analysis) has_host_root: bool,
}

#[cfg(feature = "content")]
fn content_limits(limits: &AnalysisLimits) -> crate::content::ContentLimits {
    crate::content::ContentLimits {
        max_bytes: limits.max_bytes,
        max_file_bytes: limits.max_file_bytes,
    }
}

#[cfg(feature = "content")]
fn content_import_granularity(granularity: ImportGranularity) -> crate::content::ImportGranularity {
    match granularity {
        ImportGranularity::Module => crate::content::ImportGranularity::Module,
        ImportGranularity::File => crate::content::ImportGranularity::File,
    }
}

pub(in crate::analysis) fn run(
    input: ContentInput<'_>,
    derived: &mut DerivedReport,
    outputs: &mut AnalysisOutputs,
    warnings: &mut Vec<String>,
) {
    run_todo(&input, derived, warnings);
    run_duplicate(&input, outputs, warnings);
    run_near_duplicate(&input, outputs, warnings);
    run_imports(&input, outputs, warnings);
}

fn run_todo(input: &ContentInput<'_>, derived: &mut DerivedReport, warnings: &mut Vec<String>) {
    if input.plan.todo {
        #[cfg(feature = "content")]
        if let Some(list) = input.files {
            let limits = content_limits(&input.req.limits);
            match crate::content::build_todo_report(input.root, list, &limits, derived.totals.code)
            {
                Ok(report) => derived.todo = Some(report),
                Err(err) => warnings.push(format!("todo scan failed: {}", err)),
            }
        }
        #[cfg(not(feature = "content"))]
        warnings.push(crate::grid::DisabledFeature::TodoScan.warning().to_string());
    }
}

fn run_duplicate(
    input: &ContentInput<'_>,
    outputs: &mut AnalysisOutputs,
    warnings: &mut Vec<String>,
) {
    if input.plan.dup {
        #[cfg(feature = "content")]
        if let Some(list) = input.files {
            let limits = content_limits(&input.req.limits);
            match crate::content::build_duplicate_report(input.root, list, input.export, &limits) {
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
}

fn run_near_duplicate(
    input: &ContentInput<'_>,
    outputs: &mut AnalysisOutputs,
    warnings: &mut Vec<String>,
) {
    let req = input.req;
    if req.near_dup {
        #[cfg(feature = "content")]
        {
            if input.has_host_root {
                let near_dup_limits = crate::near_dup::NearDupLimits {
                    max_bytes: req.limits.max_bytes,
                    max_file_bytes: req.limits.max_file_bytes,
                };
                match crate::near_dup::build_near_dup_report(
                    input.root,
                    input.export,
                    req.near_dup_scope,
                    req.near_dup_threshold,
                    req.near_dup_max_files,
                    req.near_dup_max_pairs,
                    &near_dup_limits,
                    &req.near_dup_exclude,
                ) {
                    Ok(report) => {
                        if let Some(ref mut dup) = outputs.dup {
                            dup.near = Some(report);
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
    }
}

fn run_imports(
    input: &ContentInput<'_>,
    outputs: &mut AnalysisOutputs,
    warnings: &mut Vec<String>,
) {
    if input.plan.imports {
        #[cfg(feature = "content")]
        if let Some(list) = input.files {
            let limits = content_limits(&input.req.limits);
            let granularity = content_import_granularity(input.req.import_granularity);
            match crate::content::build_import_report(
                input.root,
                list,
                input.export,
                granularity,
                &limits,
            ) {
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
