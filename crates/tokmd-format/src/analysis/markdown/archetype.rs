//! Archetype Markdown rendering.
//!
//! This module owns the optional project archetype classification section.

use std::fmt::Write;

use tokmd_analysis_types::Archetype;

pub(super) fn render_archetype(out: &mut String, archetype: &Archetype) {
    out.push_str("## Archetype\n\n");
    let _ = writeln!(out, "- Kind: `{}`", archetype.kind);
    if !archetype.evidence.is_empty() {
        let _ = writeln!(out, "- Evidence: `{}`", archetype.evidence.join("`, `"));
    }
    out.push('\n');
}
