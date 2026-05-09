//! Analysis input Markdown rendering.
//!
//! This module owns the caller-provided analysis input list section.

use std::fmt::Write;

pub(super) fn render_inputs(out: &mut String, inputs: &[String]) {
    out.push_str("## Inputs\n\n");
    for input in inputs {
        let _ = writeln!(out, "- `{}`", input);
    }
    out.push('\n');
}
