use super::find_brace_end_at;
use crate::complexity::functions::is_rust_fn_start;

pub(in crate::complexity) fn detect_fn_spans_rust(lines: &[&str]) -> Vec<(usize, usize, String)> {
    let mut spans = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim();
        if is_rust_fn_start(trimmed) {
            let name = extract_rust_fn_name(trimmed);
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

fn extract_rust_fn_name(line: &str) -> String {
    if let Some(idx) = line.find("fn ") {
        let after = &line[idx + 3..];
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
