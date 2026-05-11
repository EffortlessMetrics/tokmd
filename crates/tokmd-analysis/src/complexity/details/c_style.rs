//! C-family function span detection for complexity details.

use super::find_brace_end_at;

pub(in crate::complexity) fn detect_fn_spans_c_style(
    lines: &[&str],
) -> Vec<(usize, usize, String)> {
    let mut spans = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim();
        let looks_like_fn = (trimmed.ends_with(") {") || trimmed.ends_with("){"))
            && !trimmed.starts_with("if ")
            && !trimmed.starts_with("if(")
            && !trimmed.starts_with("while ")
            && !trimmed.starts_with("for ")
            && !trimmed.starts_with("switch ")
            && !trimmed.starts_with("//")
            && !trimmed.starts_with('#');
        if looks_like_fn {
            let name = extract_c_fn_name(trimmed);
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

fn extract_c_fn_name(line: &str) -> String {
    if let Some(paren_idx) = line.find('(') {
        let before = line[..paren_idx].trim();
        let name: String = before
            .chars()
            .rev()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        if !name.is_empty() {
            return name;
        }
    }
    "<unknown>".to_string()
}
