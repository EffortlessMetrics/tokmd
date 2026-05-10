//! C-family function span heuristics.

use super::{FunctionSpan, brace::find_brace_end};

/// Detect C-style functions (C, C++, Java, C#).
pub(super) fn detect_c_style_functions(lines: &[&str]) -> Vec<FunctionSpan> {
    let mut spans = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        // Heuristic: function declaration ends with `) {` or `)` followed by `{` on next line.
        let looks_like_fn = trimmed.ends_with(") {")
            || (trimmed.ends_with(')')
                && i + 1 < lines.len()
                && lines[i + 1].trim().starts_with('{'));

        // Exclude control structures.
        let is_control = trimmed.starts_with("if ")
            || trimmed.starts_with("if(")
            || trimmed.starts_with("while ")
            || trimmed.starts_with("while(")
            || trimmed.starts_with("for ")
            || trimmed.starts_with("for(")
            || trimmed.starts_with("switch ")
            || trimmed.starts_with("switch(")
            || trimmed.starts_with("catch ")
            || trimmed.starts_with("catch(");

        if looks_like_fn && !is_control {
            let start = i;
            if let Some(end) = find_brace_end(lines, i) {
                spans.push(FunctionSpan {
                    start_line: start,
                    end_line: end,
                });
                i = end + 1;
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }

    spans
}
