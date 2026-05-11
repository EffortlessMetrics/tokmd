//! JavaScript and TypeScript function span detection for complexity details.

use super::find_brace_end_at;

pub(in crate::complexity) fn detect_fn_spans_js(lines: &[&str]) -> Vec<(usize, usize, String)> {
    let mut spans = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim();
        let is_fn = trimmed.starts_with("function ")
            || trimmed.starts_with("async function ")
            || trimmed.starts_with("export function ")
            || trimmed.starts_with("export async function ")
            || trimmed.contains("=> {");
        if is_fn && !trimmed.starts_with("//") {
            let name = extract_js_fn_name(trimmed);
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

fn extract_js_fn_name(line: &str) -> String {
    if let Some(idx) = line.find("function ") {
        let after = &line[idx + 9..];
        let name: String = after
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '$')
            .collect();
        if !name.is_empty() {
            return name;
        }
    }
    if let Some(paren_idx) = line.find('(') {
        let before = line[..paren_idx].trim();
        let name: String = before
            .chars()
            .rev()
            .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '$')
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        if !name.is_empty() {
            return name;
        }
    }
    "<anonymous>".to_string()
}
