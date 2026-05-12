use std::path::PathBuf;

use crate::grid::PresetPlan;

#[cfg(feature = "walk")]
use super::warnings;
use super::{AnalysisContext, AnalysisRequest};

pub(super) fn collect_analysis_files(
    ctx: &AnalysisContext,
    req: &AnalysisRequest,
    plan: &PresetPlan,
    warnings_out: &mut Vec<String>,
) -> Option<Vec<PathBuf>> {
    if !plan.needs_files() {
        return None;
    }

    #[cfg(feature = "walk")]
    {
        if warnings::has_host_root(&ctx.root) {
            match tokmd_scan::walk::list_files(&ctx.root, req.limits.max_files) {
                Ok(list) => Some(list),
                Err(err) => {
                    warnings_out.push(format!("walk failed: {}", err));
                    None
                }
            }
        } else {
            warnings::push_warning_once(warnings_out, warnings::ROOTLESS_FILE_ANALYSIS_WARNING);
            None
        }
    }

    #[cfg(not(feature = "walk"))]
    {
        let _ = (ctx, req);
        warnings_out.push(
            crate::grid::DisabledFeature::FileInventory
                .warning()
                .to_string(),
        );
        None
    }
}
