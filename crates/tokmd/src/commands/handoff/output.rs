//! Handoff bundle output writers.
//!
//! Output generation is split by artifact responsibility so the handoff command
//! can keep payload writing, external evidence links, and work-order rendering
//! independently testable.

use std::path::Path;

mod artifacts;
mod links;
mod work_order;

pub(super) use artifacts::{write_manifest_json, write_payloads};
pub(super) use links::{HandoffLinkInputs, write_link_artifacts};
pub(super) use work_order::{HandoffWorkOrderInputs, write_work_order};

fn has_any_link(links: &HandoffLinkInputs<'_>) -> bool {
    links.review_packet_dir.is_some()
        || links.review_packet_check.is_some()
        || links.affected.is_some()
        || links.proof_plan.is_some()
}

fn path_string(path: &Path) -> String {
    path.display().to_string().replace('\\', "/")
}
