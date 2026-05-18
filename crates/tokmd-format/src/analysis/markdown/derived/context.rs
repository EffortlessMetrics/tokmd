//! Context-window rendering for derived analysis reports.

use std::fmt::Write;

use tokmd_analysis_types::DerivedReport;

use crate::analysis::markdown::fmt_pct;

pub(super) fn render_context_sections(out: &mut String, derived: &DerivedReport) {
    if let Some(context) = &derived.context_window {
        out.push_str("## Context window\n\n");
        let _ = writeln!(
            out,
            "- Window tokens: `{}`\n- Total tokens: `{}`\n- Utilization: `{}`\n- Fits: `{}`\n",
            context.window_tokens,
            context.total_tokens,
            fmt_pct(context.pct),
            context.fits
        );
    }
}
