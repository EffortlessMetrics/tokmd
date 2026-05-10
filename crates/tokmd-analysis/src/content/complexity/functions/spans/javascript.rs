//! JavaScript and TypeScript function span heuristics.

use super::{
    FunctionSpan,
    brace::find_brace_end,
    patterns::{JS_ARROW, JS_FUNCTION, JS_METHOD},
};

/// Detect functions in JavaScript/TypeScript.
pub(super) fn detect_js_functions(lines: &[&str]) -> Vec<FunctionSpan> {
    let mut spans = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        if JS_FUNCTION.is_match(line) || JS_ARROW.is_match(line) || JS_METHOD.is_match(line) {
            // Avoid matching control structures like if(...) {
            if is_likely_function_start(line) {
                let start = i;
                if let Some(end) = find_brace_end(lines, i) {
                    spans.push(FunctionSpan {
                        start_line: start,
                        end_line: end,
                    });
                    i = end + 1;
                    continue;
                }
            }
        }
        i += 1;
    }

    spans
}

/// Check if a line is likely the start of an actual function, not a method call.
fn is_likely_function_start(line: &str) -> bool {
    let trimmed = line.trim();
    !trimmed.starts_with("//")
        && !trimmed.starts_with("/*")
        && !trimmed.starts_with('*')
        && !trimmed.ends_with(',')
        && !trimmed.ends_with(';')
}
