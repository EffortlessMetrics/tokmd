//! License Markdown rendering.
//!
//! This module owns license radar summary and finding table rendering for
//! analysis Markdown output.

use std::fmt::Write;

use super::fmt_f64;
use tokmd_analysis_types::LicenseReport;

pub(super) fn render_license_report(out: &mut String, license: &LicenseReport) {
    out.push_str("## License radar\n\n");
    if let Some(effective) = &license.effective {
        let _ = writeln!(out, "- Effective: `{}`", effective);
    }
    out.push_str("- Heuristic detection; not legal advice.\n\n");
    if !license.findings.is_empty() {
        out.push_str("|SPDX|Confidence|Source|Kind|\n");
        out.push_str("|---|---:|---|---|\n");
        for row in license.findings.iter().take(10) {
            let _ = writeln!(
                out,
                "|{}|{}|{}|{:?}|",
                row.spdx,
                fmt_f64(row.confidence as f64, 2),
                row.source_path,
                row.source_kind
            );
        }
        out.push('\n');
    }
}
