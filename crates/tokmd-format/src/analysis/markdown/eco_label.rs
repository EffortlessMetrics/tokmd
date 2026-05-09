//! Eco label Markdown rendering.
//!
//! This module owns the optional fun eco-label section for analysis Markdown.

use std::fmt::Write;

use super::fmt_f64;
use tokmd_analysis_types::EcoLabel;

pub(super) fn render_eco_label(out: &mut String, label: &EcoLabel) {
    out.push_str("## Eco label\n\n");
    let _ = writeln!(
        out,
        "- Label: `{}`\n- Score: `{}`\n- Bytes: `{}`\n- Notes: `{}`\n",
        label.label,
        fmt_f64(label.score, 1),
        label.bytes,
        label.notes
    );
}
