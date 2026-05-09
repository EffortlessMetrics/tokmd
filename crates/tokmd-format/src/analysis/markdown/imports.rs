//! Import graph Markdown rendering.
//!
//! This module owns the imports section and its truncated edge table for
//! analysis Markdown output.

use std::fmt::Write;

use tokmd_analysis_types::ImportReport;

pub(super) fn render_import_report(out: &mut String, imports: &ImportReport) {
    out.push_str("## Imports\n\n");
    let _ = writeln!(out, "- Granularity: `{}`\n", imports.granularity);
    if !imports.edges.is_empty() {
        out.push_str("|From|To|Count|\n");
        out.push_str("|---|---|---:|\n");
        for row in imports.edges.iter().take(20) {
            let _ = writeln!(out, "|{}|{}|{}|", row.from, row.to, row.count);
        }
        out.push('\n');
    }
}
