//! Dependency Markdown rendering.
//!
//! This module owns dependency totals and lockfile rows for analysis Markdown
//! output.

use std::fmt::Write;

use tokmd_analysis_types::DependencyReport;

pub(super) fn render_dependency_report(out: &mut String, deps: &DependencyReport) {
    out.push_str("## Dependencies\n\n");
    let _ = writeln!(out, "- Total: `{}`\n", deps.total);
    if !deps.lockfiles.is_empty() {
        out.push_str("|Lockfile|Kind|Dependencies|\n");
        out.push_str("|---|---|---:|\n");
        for row in &deps.lockfiles {
            let _ = writeln!(out, "|{}|{}|{}|", row.path, row.kind, row.dependencies);
        }
        out.push('\n');
    }
}
