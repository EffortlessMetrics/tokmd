//! Complexity Markdown rendering.
//!
//! This module owns the complexity metrics table and top-complex-file detail
//! rendering for analysis Markdown output.

use std::fmt::Write;

use super::fmt_f64;
use tokmd_analysis_types::ComplexityReport;

pub(super) fn render_complexity_report(out: &mut String, cx: &ComplexityReport) {
    out.push_str("## Complexity\n\n");
    out.push_str("|Metric|Value|\n");
    out.push_str("|---|---:|\n");
    let _ = writeln!(out, "|Total functions|{}|", cx.total_functions);
    let _ = writeln!(
        out,
        "|Avg function length|{}|",
        fmt_f64(cx.avg_function_length, 1)
    );
    let _ = writeln!(out, "|Max function length|{}|", cx.max_function_length);
    let _ = writeln!(out, "|Avg cyclomatic|{}|", fmt_f64(cx.avg_cyclomatic, 2));
    let _ = writeln!(out, "|Max cyclomatic|{}|", cx.max_cyclomatic);
    if let Some(cog) = cx.avg_cognitive {
        let _ = writeln!(out, "|Avg cognitive|{}|", fmt_f64(cog, 2));
    }
    if let Some(cog) = cx.max_cognitive {
        let _ = writeln!(out, "|Max cognitive|{}|", cog);
    }
    if let Some(avg_nesting) = cx.avg_nesting_depth {
        let _ = writeln!(out, "|Avg nesting depth|{}|", fmt_f64(avg_nesting, 2));
    }
    if let Some(max_nesting) = cx.max_nesting_depth {
        let _ = writeln!(out, "|Max nesting depth|{}|", max_nesting);
    }
    let _ = writeln!(out, "|High risk files|{}|\n", cx.high_risk_files);

    if !cx.files.is_empty() {
        out.push_str("### Top complex files\n\n");
        out.push_str("|Path|CC|Functions|Max fn length|\n");
        out.push_str("|---|---:|---:|---:|\n");
        for f in cx.files.iter().take(10) {
            let _ = writeln!(
                out,
                "|{}|{}|{}|{}|",
                f.path, f.cyclomatic_complexity, f.function_count, f.max_function_length
            );
        }
        out.push('\n');
    }
}
