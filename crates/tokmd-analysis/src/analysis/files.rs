#![allow(dead_code, unused_variables)]

use std::path::{Path, PathBuf};

use crate::grid::PresetPlan;
use tokmd_analysis_types::AnalysisLimits;

const ROOTLESS_FILE_ANALYSIS_WARNING: &str =
    "in-memory analysis has no host root; skipping file-backed enrichers";
pub(super) const ROOTLESS_GIT_ANALYSIS_WARNING: &str =
    "in-memory analysis has no host root; skipping git-backed enrichers";

pub(super) fn has_host_root(root: &Path) -> bool {
    !root.as_os_str().is_empty()
}

pub(super) fn push_warning_once(warnings: &mut Vec<String>, warning: &str) {
    if warnings.iter().all(|existing| existing != warning) {
        warnings.push(warning.to_string());
    }
}

pub(super) fn collect_required_files(
    root: &Path,
    limits: &AnalysisLimits,
    plan: &PresetPlan,
    has_host_root: bool,
    warnings: &mut Vec<String>,
) -> Option<Vec<PathBuf>> {
    if !plan.needs_files() {
        return None;
    }

    #[cfg(feature = "walk")]
    {
        if has_host_root {
            match tokmd_scan::walk::list_files(root, limits.max_files) {
                Ok(list) => Some(list),
                Err(err) => {
                    warnings.push(format!("walk failed: {}", err));
                    None
                }
            }
        } else {
            push_warning_once(warnings, ROOTLESS_FILE_ANALYSIS_WARNING);
            None
        }
    }

    #[cfg(not(feature = "walk"))]
    {
        let _ = (root, limits, has_host_root);
        warnings.push(
            crate::grid::DisabledFeature::FileInventory
                .warning()
                .to_string(),
        );
        None
    }
}
