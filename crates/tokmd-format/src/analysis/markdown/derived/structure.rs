//! Structural metric rendering for derived analysis reports.

use std::fmt::Write;

use tokmd_analysis_types::DerivedReport;

use crate::analysis::markdown::fmt_f64;

pub(super) fn render_structure(out: &mut String, derived: &DerivedReport) {
    out.push_str("## Structure\n\n");
    let _ = writeln!(
        out,
        "- Max depth: `{}`\n- Avg depth: `{}`\n",
        derived.nesting.max,
        fmt_f64(derived.nesting.avg, 2)
    );
}
