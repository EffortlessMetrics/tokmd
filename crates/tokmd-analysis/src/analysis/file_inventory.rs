use std::path::{Path, PathBuf};

use crate::grid::PresetPlan;
use tokmd_analysis_types::AnalysisLimits;

#[cfg(feature = "walk")]
use super::{ROOTLESS_FILE_ANALYSIS_WARNING, has_host_root, push_warning_once};

pub(super) fn collect_files_for_plan(
    root: &Path,
    limits: &AnalysisLimits,
    plan: &PresetPlan,
    warnings: &mut Vec<String>,
) -> Option<Vec<PathBuf>> {
    if !plan.needs_files() {
        return None;
    }

    #[cfg(feature = "walk")]
    {
        if has_host_root(root) {
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
        let _ = (root, limits);
        warnings.push(
            crate::grid::DisabledFeature::FileInventory
                .warning()
                .to_string(),
        );
        None
    }
}
