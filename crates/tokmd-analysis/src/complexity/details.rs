//! Function-level complexity detail extraction.

use tokmd_analysis_types::FunctionComplexityDetail;

use super::functions::is_rust_fn_start;
use super::map_language_for_complexity;

pub(super) fn extract_function_details(lang: &str, text: &str) -> Vec<FunctionComplexityDetail> {
    let lines: Vec<&str> = text.lines().collect();
    let mapped_lang = map_language_for_complexity(lang);

    let fn_spans: Vec<(usize, usize, String)> = match lang.to_lowercase().as_str() {
        "rust" => detect_fn_spans_rust(&lines),
        "javascript" | "typescript" => detect_fn_spans_js(&lines),
        "python" => detect_fn_spans_python(&lines),
        "go" => detect_fn_spans_go(&lines),
        "c" | "c++" | "java" | "c#" | "php" => detect_fn_spans_c_style(&lines),
        _ => Vec::new(),
    };

    fn_spans
        .into_iter()
        .map(|(start, end, name)| {
            let length = end.saturating_sub(start) + 1;
            let fn_text = lines[start..=end.min(lines.len() - 1)].join("\n");
            let cyclomatic = estimate_cyclomatic_inline(mapped_lang, &fn_text);

            let cognitive_result =
                crate::content::complexity::estimate_cognitive_complexity(&fn_text, mapped_lang);
            let cognitive = Some(cognitive_result.total);

            let nesting_result =
                crate::content::complexity::analyze_nesting_depth(&fn_text, mapped_lang);
            let max_nesting = if nesting_result.max_depth > 0 {
                Some(nesting_result.max_depth)
            } else {
                None
            };

            let param_count = count_params(lines.get(start).unwrap_or(&""));

            FunctionComplexityDetail {
                name,
                line_start: start + 1,
                line_end: end + 1,
                length,
                cyclomatic,
                cognitive,
                max_nesting,
                param_count: if param_count > 0 {
                    Some(param_count)
                } else {
                    None
                },
            }
        })
        .collect()
}

pub(super) fn detect_fn_spans_rust(lines: &[&str]) -> Vec<(usize, usize, String)> {
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

fn detect_fn_spans_js(lines: &[&str]) -> Vec<(usize, usize, String)> {
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

pub(super) fn detect_fn_spans_python(lines: &[&str]) -> Vec<(usize, usize, String)> {
    let mut spans = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim();
        if trimmed.starts_with("def ") || trimmed.starts_with("async def ") {
            let name = extract_python_fn_name(trimmed);
            let base_indent = lines[i].len() - lines[i].trim_start().len();

            let mut start = i;
            {
                let mut k = i;
                while k > 0 {
                    let prev_line = lines[k - 1];
                    let prev_trimmed = prev_line.trim();

                    if prev_trimmed.is_empty() {
                        k -= 1;
                        continue;
                    }

                    if prev_trimmed.starts_with('#') {
                        k -= 1;
                        continue;
                    }

                    let prev_indent = prev_line.len() - prev_line.trim_start().len();
                    if prev_indent == base_indent && prev_trimmed.starts_with('@') {
                        start = k - 1;
                        k -= 1;
                    } else {
                        break;
                    }
                }
            }
            let mut end = i;
            let mut j = i + 1;
            while j < lines.len() {
                let lt = lines[j].trim();
                if lt.is_empty() || lt.starts_with('#') {
                    j += 1;
                    continue;
                }
                let indent = lines[j].len() - lines[j].trim_start().len();
                if indent <= base_indent {
                    break;
                }
                end = j;
                j += 1;
            }
            spans.push((start, end, name));
            i = end + 1;
        } else {
            i += 1;
        }
    }
    spans
}

fn detect_fn_spans_go(lines: &[&str]) -> Vec<(usize, usize, String)> {
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

pub(super) fn detect_fn_spans_c_style(lines: &[&str]) -> Vec<(usize, usize, String)> {
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

fn find_brace_end_at(lines: &[&str], start_line: usize) -> Option<usize> {
    let mut depth: usize = 0;
    let mut found_open = false;
    for (i, line) in lines.iter().enumerate().skip(start_line) {
        for ch in line.chars() {
            if ch == '{' {
                depth += 1;
                found_open = true;
            } else if ch == '}' {
                depth = depth.saturating_sub(1);
                if found_open && depth == 0 {
                    return Some(i);
                }
            }
        }
    }
    None
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

fn extract_python_fn_name(line: &str) -> String {
    let keyword = if line.contains("async def ") {
        "async def "
    } else {
        "def "
    };
    if let Some(idx) = line.find(keyword) {
        let after = &line[idx + keyword.len()..];
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

fn count_params(line: &str) -> usize {
    if let Some(open) = line.find('(')
        && let Some(close) = line.find(')')
    {
        let params = line[open + 1..close].trim();
        if params.is_empty() {
            return 0;
        }
        return params.split(',').count();
    }
    0
}

fn estimate_cyclomatic_inline(lang: &str, text: &str) -> usize {
    let mut complexity = 1usize;
    let keywords: &[&str] = match lang {
        "rust" => &["if ", "match ", "while ", "for ", "loop ", "?", "&&", "||"],
        "javascript" | "typescript" => {
            &["if ", "case ", "while ", "for ", "?", "&&", "||", "catch "]
        }
        "python" => &["if ", "elif ", "while ", "for ", "except ", " and ", " or "],
        "go" => &["if ", "case ", "for ", "select ", "&&", "||"],
        "c" | "c++" | "java" | "c#" | "php" => {
            &["if ", "case ", "while ", "for ", "?", "&&", "||", "catch "]
        }
        _ => &[],
    };
    let lower = text.to_lowercase();
    for keyword in keywords {
        complexity += lower.matches(keyword).count();
    }
    complexity
}
