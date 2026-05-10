//! Indentation-delimited function span helpers.

use regex::Regex;

use super::super::super::shared::get_indent;
use super::FunctionSpan;

/// Detect functions in indentation-based languages (Python).
pub(super) fn detect_indented_functions(lines: &[&str], pattern: &Regex) -> Vec<FunctionSpan> {
    let mut spans = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        if pattern.is_match(lines[i]) {
            let mut start = i;
            let base_indent = get_indent(lines[i]);

            // Walk upward to include decorator lines at the same indent level.
            // Skip blank lines only tentatively; commit only if a decorator is found.
            {
                let mut probe = start;
                while probe > 0 {
                    let prev = lines[probe - 1].trim();
                    if prev.is_empty() {
                        probe -= 1;
                        continue;
                    }
                    let prev_indent = get_indent(lines[probe - 1]);
                    if prev_indent == base_indent && prev.starts_with('@') {
                        probe -= 1;
                        start = probe;
                    } else {
                        break;
                    }
                }
            }

            let end = find_indent_end(lines, i, base_indent);
            spans.push(FunctionSpan {
                start_line: start,
                end_line: end,
            });
            i = end + 1;
        } else {
            i += 1;
        }
    }

    spans
}

/// Find the end of an indented block.
fn find_indent_end(lines: &[&str], start_line: usize, base_indent: usize) -> usize {
    let mut last_content_line = start_line;

    for (i, line) in lines.iter().enumerate().skip(start_line + 1) {
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let indent = get_indent(line);
        if indent <= base_indent {
            return last_content_line;
        }

        last_content_line = i;
    }

    last_content_line
}
