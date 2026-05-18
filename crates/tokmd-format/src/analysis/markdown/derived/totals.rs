//! Totals table rendering for derived analysis reports.

use std::fmt::Write;

use tokmd_analysis_types::DerivedReport;

pub(super) fn render_totals(out: &mut String, derived: &DerivedReport) {
    out.push_str("## Totals\n\n");
    out.push_str("|Files|Code|Comments|Blanks|Lines|Bytes|Tokens|\n");
    out.push_str("|---:|---:|---:|---:|---:|---:|---:|\n");
    let _ = writeln!(
        out,
        "|{}|{}|{}|{}|{}|{}|{}|\n",
        derived.totals.files,
        derived.totals.code,
        derived.totals.comments,
        derived.totals.blanks,
        derived.totals.lines,
        derived.totals.bytes,
        derived.totals.tokens
    );
}
