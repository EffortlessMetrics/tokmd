//! Function-level complexity detail extraction.

use tokmd_analysis_types::FunctionComplexityDetail;

use super::map_language_for_complexity;

mod c_style;
mod go;
mod javascript;
mod python;
mod rust;

pub(super) use c_style::detect_fn_spans_c_style;
pub(super) use go::detect_fn_spans_go;
pub(super) use javascript::detect_fn_spans_js;
pub(super) use python::detect_fn_spans_python;
pub(super) use rust::detect_fn_spans_rust;

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
