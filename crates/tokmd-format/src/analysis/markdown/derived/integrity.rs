//! Integrity section rendering for derived analysis reports.

use std::fmt::Write;

use tokmd_analysis_types::DerivedReport;

pub(super) fn render_integrity(out: &mut String, derived: &DerivedReport) {
    out.push_str("## Integrity\n\n");
    let _ = writeln!(
        out,
        "- Hash: `{}` (`{}`)\n- Entries: `{}`\n",
        derived.integrity.hash, derived.integrity.algo, derived.integrity.entries
    );
}
