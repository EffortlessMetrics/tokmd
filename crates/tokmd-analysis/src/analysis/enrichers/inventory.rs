#![cfg_attr(not(feature = "walk"), allow(unused_variables, clippy::ptr_arg))]
use std::path::{Path, PathBuf};

use crate::grid::PresetPlan;

use super::super::outputs::AnalysisOutputs;

pub(in crate::analysis) fn run(
    root: &Path,
    files: Option<&[PathBuf]>,
    plan: &PresetPlan,
    outputs: &mut AnalysisOutputs,
    warnings: &mut Vec<String>,
) {
    if plan.assets {
        #[cfg(feature = "walk")]
        if let Some(list) = files {
            match crate::assets::build_assets_report(root, list) {
                Ok(report) => outputs.assets = Some(report),
                Err(err) => warnings.push(format!("asset scan failed: {}", err)),
            }
        }
    }

    if plan.deps {
        #[cfg(feature = "walk")]
        if let Some(list) = files {
            match crate::assets::build_dependency_report(root, list) {
                Ok(report) => outputs.deps = Some(report),
                Err(err) => warnings.push(format!("dependency scan failed: {}", err)),
            }
        }
    }
}
