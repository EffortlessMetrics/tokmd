//! Entropy Markdown rendering.
//!
//! This module owns entropy outlier table rendering for analysis Markdown output.

use std::fmt::Write;

use super::fmt_f64;
use tokmd_analysis_types::EntropyReport;

pub(super) fn render_entropy_report(out: &mut String, entropy: &EntropyReport) {
    out.push_str("## Entropy profiling\n\n");
    if entropy.suspects.is_empty() {
        out.push_str("- No entropy outliers detected.\n\n");
    } else {
        out.push_str("|Path|Module|Entropy|Sample bytes|Class|\n");
        out.push_str("|---|---|---:|---:|---|\n");
        for row in entropy.suspects.iter().take(10) {
            let _ = writeln!(
                out,
                "|{}|{}|{}|{}|{:?}|",
                row.path,
                row.module,
                fmt_f64(row.entropy_bits_per_byte as f64, 2),
                row.sample_bytes,
                row.class
            );
        }
        out.push('\n');
    }
}
