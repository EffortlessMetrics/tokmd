//! Go function span detection for complexity details.

use super::find_brace_end_at;

pub(in crate::complexity) fn detect_fn_spans_go(lines: &[&str]) -> Vec<(usize, usize, String)> {
    let mut spans = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim();
        if trimmed.starts_with("func ") {
            let name = extract_go_fn_name(trimmed);
            let start = i;
            if let Some(end) = find_brace_end_at(lines, i) {
                spans.push((start, end, name));
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

fn extract_go_fn_name(line: &str) -> String {
    if let Some(idx) = line.find("func ") {
        let after = &line[idx + 5..];
        let after = if after.starts_with('(') {
            if let Some(close) = after.find(')') {
                after[close + 1..].trim_start()
            } else {
                after
            }
        } else {
            after
        };
        let name: String = after
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if !name.is_empty() {
            return name;
        }
    }
    "<unknown>".to_string()
}
