//! Mermaid rendering for analysis receipts.
//!
//! This module owns the small graph projection used by
//! `AnalysisFormat::Mermaid`; analysis computation and import discovery stay
//! in the analysis crates.

use std::fmt::Write;

use tokmd_analysis_types::AnalysisReceipt;

pub(super) fn render(receipt: &AnalysisReceipt) -> String {
    let mut out = String::from("graph TD\n");
    if let Some(imports) = &receipt.imports {
        for edge in imports.edges.iter().take(200) {
            let from = sanitize_node_name(&edge.from);
            let to = sanitize_node_name(&edge.to);
            let _ = writeln!(out, "  {} -->|{}| {}", from, edge.count, to);
        }
    }
    out
}

fn sanitize_node_name(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}
