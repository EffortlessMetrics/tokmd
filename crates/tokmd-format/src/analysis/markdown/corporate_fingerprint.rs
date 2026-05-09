//! Corporate fingerprint Markdown rendering.
//!
//! This module owns commit-domain summary rendering for analysis Markdown output.

use std::fmt::Write;

use super::fmt_pct;
use tokmd_analysis_types::CorporateFingerprint;

pub(super) fn render_corporate_fingerprint(out: &mut String, fingerprint: &CorporateFingerprint) {
    out.push_str("## Corporate fingerprint\n\n");
    if fingerprint.domains.is_empty() {
        out.push_str("- No commit domains detected.\n\n");
    } else {
        out.push_str("|Domain|Commits|Pct|\n");
        out.push_str("|---|---:|---:|\n");
        for row in fingerprint.domains.iter().take(10) {
            let _ = writeln!(
                out,
                "|{}|{}|{}|",
                row.domain,
                row.commits,
                fmt_pct(row.pct as f64)
            );
        }
        out.push('\n');
    }
}
