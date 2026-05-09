//! Asset Markdown rendering.
//!
//! This module owns the asset totals, category breakdown, and top-file tables
//! for analysis Markdown output.

use std::fmt::Write;

use tokmd_analysis_types::AssetReport;

pub(super) fn render_asset_report(out: &mut String, assets: &AssetReport) {
    out.push_str("## Assets\n\n");
    let _ = writeln!(
        out,
        "- Total files: `{}`\n- Total bytes: `{}`\n",
        assets.total_files, assets.total_bytes
    );
    if !assets.categories.is_empty() {
        out.push_str("|Category|Files|Bytes|Extensions|\n");
        out.push_str("|---|---:|---:|---|\n");
        for row in &assets.categories {
            let _ = writeln!(
                out,
                "|{}|{}|{}|{}|",
                row.category,
                row.files,
                row.bytes,
                row.extensions.join(", ")
            );
        }
        out.push('\n');
    }
    if !assets.top_files.is_empty() {
        out.push_str("|File|Bytes|Category|\n");
        out.push_str("|---|---:|---|\n");
        for row in &assets.top_files {
            let _ = writeln!(out, "|{}|{}|{}|", row.path, row.bytes, row.category);
        }
        out.push('\n');
    }
}
