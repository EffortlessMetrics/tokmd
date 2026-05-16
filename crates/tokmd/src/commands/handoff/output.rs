//! Handoff bundle output writers.
//!
//! Artifact writing, linked-evidence parsing, and work-order rendering are split
//! into focused submodules so each output concern can evolve independently.

use std::path::Path;

use tokmd_types::{ArtifactEntry, ContextFileRow};

mod artifact;
mod evidence;
mod links;
mod work_order;

pub(super) use artifact::{write_manifest_json, write_payloads};
pub(super) use links::write_link_artifacts;
pub(super) use work_order::write_work_order;

pub(super) struct HandoffPayloads {
    pub(super) map_bytes: u64,
    pub(super) intelligence_bytes: u64,
    pub(super) code_bytes: u64,
    pub(super) artifacts: Vec<ArtifactEntry>,
}

pub(super) struct HandoffLinkInputs<'a> {
    pub(super) review_packet_dir: Option<&'a Path>,
    pub(super) review_packet_check: Option<&'a Path>,
    pub(super) affected: Option<&'a Path>,
    pub(super) proof_plan: Option<&'a Path>,
}

pub(super) struct HandoffWorkOrderInputs<'a> {
    pub(super) inputs: &'a [String],
    pub(super) budget_tokens: usize,
    pub(super) used_tokens: usize,
    pub(super) utilization_pct: f64,
    pub(super) strategy: &'a str,
    pub(super) rank_by: &'a str,
    pub(super) intelligence_preset: &'a str,
    pub(super) total_files: usize,
    pub(super) selected: &'a [ContextFileRow],
    pub(super) links: &'a HandoffLinkInputs<'a>,
}
