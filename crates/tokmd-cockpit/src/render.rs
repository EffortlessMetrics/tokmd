//! Rendering functions for cockpit receipts.
//!
//! Provides JSON, Markdown, sections, comment, and review packet output formats.

use anyhow::{Context, Result};

use crate::CockpitReceipt;

mod artifacts;
mod comment;
mod evidence;
mod manifest;
mod markdown;
mod proof_summary;
mod review_map;
mod review_map_proof;
mod review_packet;
mod sections;

pub use artifacts::write_artifacts;
#[cfg(feature = "git")]
pub use artifacts::write_sensor_artifacts;
pub use comment::render_comment_md;
pub use markdown::render_markdown;
pub use review_packet::{
    write_review_packet, write_review_packet_with_imported_evidence,
    write_review_packet_with_proof_evidence,
};
pub use sections::render_sections;

/// Render receipt as JSON.
pub fn render_json(receipt: &CockpitReceipt) -> Result<String> {
    serde_json::to_string_pretty(receipt).context("Failed to serialize receipt to JSON")
}
