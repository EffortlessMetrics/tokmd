//! Brace-delimited function span helpers.

use regex::Regex;

use super::FunctionSpan;

/// Detect functions in brace-based languages (Rust, Go).
pub(super) fn detect_brace_functions(lines: &[&str], pattern: &Regex) -> Vec<FunctionSpan> {
    let mut spans = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        if pattern.is_match(lines[i]) {
            let start = i;
            if let Some(end) = find_brace_end(lines, i) {
                spans.push(FunctionSpan {
                    start_line: start,
                    end_line: end,
                });
                i = end + 1;
            } else {
                // No body found (trait sig, abstract, extern) -- skip.
                i += 1;
            }
        } else {
            i += 1;
        }
    }

    spans
}

/// Find the closing brace for a function starting at `start_line`.
///
/// Returns `None` if no opening brace is found, such as trait method
/// signatures, extern declarations, or abstract methods.
pub(super) fn find_brace_end(lines: &[&str], start_line: usize) -> Option<usize> {
    let mut brace_count: usize = 0;
    let mut found_open = false;

    for (i, line) in lines.iter().enumerate().skip(start_line) {
        for ch in line.chars() {
            if ch == '{' {
                brace_count += 1;
                found_open = true;
            } else if ch == '}' {
                brace_count = brace_count.saturating_sub(1);
                if found_open && brace_count == 0 {
                    return Some(i);
                }
            }
        }
    }

    None
}
