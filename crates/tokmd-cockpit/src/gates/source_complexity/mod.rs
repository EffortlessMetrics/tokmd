//! Source-level complexity helpers shared by review-oriented surfaces.
//!
//! These helpers are intentionally lightweight and heuristic. They preserve
//! function-scoped Rust complexity for cockpit review gates without pulling in
//! the full analysis preset pipeline or changing receipt schemas.

mod mask;

use mask::RustCodeMask;

/// Summary of function-scoped Rust complexity for one source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RustFunctionComplexitySummary {
    /// Total cyclomatic complexity across all detected functions.
    pub total_complexity: u32,
    /// Maximum complexity of any single detected function.
    pub max_complexity: u32,
    /// Number of functions detected in the source file.
    pub function_count: usize,
    /// Maximum detected function length in lines.
    pub max_function_length: usize,
}

/// Analyze function-scoped cyclomatic complexity of Rust source code.
///
/// The keyword list intentionally omits `else if`: an `else if` branch already
/// contains `if`, and counting both terms double-counts a single decision.
pub fn analyze_rust_function_complexity(content: &str) -> RustFunctionComplexitySummary {
    let mut total_complexity: u32 = 0;
    let mut max_complexity: u32 = 0;
    let mut function_count: usize = 0;
    let mut max_function_length: usize = 0;

    let mut in_function = false;
    let mut brace_depth: i32 = 0;
    let mut function_brace_depth: i32 = 0;
    let mut function_start_line: usize = 0;
    let mut current_complexity: u32 = 1;
    let mut mask = RustCodeMask::default();

    for (line_idx, line) in content.lines().enumerate() {
        let code_line = mask.code_only_line(line);
        let trimmed = code_line.trim();

        if trimmed.is_empty() {
            continue;
        }

        let is_fn_start = !in_function && is_rust_fn_start(trimmed);

        if is_fn_start {
            in_function = true;
            function_start_line = line_idx;
            function_brace_depth = brace_depth;
            current_complexity = 1;
        }

        for c in code_line.chars() {
            if c == '{' {
                brace_depth += 1;
            } else if c == '}' {
                brace_depth -= 1;
                if in_function && brace_depth == function_brace_depth {
                    let function_length = line_idx - function_start_line + 1;
                    max_function_length = max_function_length.max(function_length);
                    total_complexity += current_complexity;
                    max_complexity = max_complexity.max(current_complexity);
                    function_count += 1;
                    in_function = false;
                    current_complexity = 1;
                }
            }
        }

        if in_function {
            for kw in ["if ", "while ", "for ", "loop ", "match ", "&&", "||", "?"] {
                let mut search_line = trimmed;
                while let Some(pos) = search_line.find(kw) {
                    current_complexity += 1;
                    search_line = &search_line[pos + kw.len()..];
                }
            }

            current_complexity += trimmed.matches("=>").count() as u32;
        }
    }

    if in_function {
        function_count += 1;
        total_complexity += current_complexity;
        max_complexity = max_complexity.max(current_complexity);
    }

    RustFunctionComplexitySummary {
        total_complexity,
        max_complexity,
        function_count,
        max_function_length,
    }
}

fn is_rust_fn_start(trimmed: &str) -> bool {
    let Some(fn_pos) = trimmed.find("fn ") else {
        return false;
    };

    let mut rest = trimmed[..fn_pos].trim();
    if rest.is_empty() {
        return true;
    }

    while !rest.is_empty() {
        rest = rest.trim_start();
        if rest.is_empty() {
            break;
        }
        if rest.starts_with("pub(") {
            let Some(close) = rest.find(')') else {
                return false;
            };
            rest = &rest[close + 1..];
        } else if let Some(next) = rest.strip_prefix("pub") {
            rest = next;
        } else if let Some(next) = rest.strip_prefix("async") {
            rest = next;
        } else if let Some(next) = rest.strip_prefix("unsafe") {
            rest = next;
        } else if let Some(next) = rest.strip_prefix("const") {
            rest = next;
        } else if rest.starts_with("extern") {
            rest = rest["extern".len()..].trim_start();
            if rest.starts_with('"') {
                let Some(close) = rest[1..].find('"') else {
                    return false;
                };
                rest = &rest[close + 2..];
            }
        } else {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::analyze_rust_function_complexity;

    #[test]
    fn rust_complexity_counts_else_if_once() {
        let analysis = analyze_rust_function_complexity(
            r#"
fn branchy(x: i32) -> i32 {
    if x > 0 {
        1
    } else if x < 0 {
        -1
    } else if x == 0 {
        0
    } else {
        42
    }
}
"#,
        );

        assert_eq!(analysis.function_count, 1);
        assert_eq!(analysis.total_complexity, 4);
        assert_eq!(analysis.max_complexity, 4);
    }

    #[test]
    fn rust_complexity_ignores_decisions_in_strings_and_comments() {
        let analysis = analyze_rust_function_complexity(
            r###"
fn only_real_branch(flag: bool) {
    let _normal = "if while for loop match && || ? => { }";
    let _raw = r##"if while for loop match && || ? => { }"##;
    let _char = '?';
    /* if outer /* while nested */ match ignored => */
    if flag {
        println!("ok"); // else if ignored && ||
    }
}
"###,
        );

        assert_eq!(analysis.function_count, 1);
        assert_eq!(analysis.total_complexity, 2);
        assert_eq!(analysis.max_complexity, 2);
    }

    #[test]
    fn rust_complexity_counts_code_before_trailing_comment() {
        let analysis = analyze_rust_function_complexity(
            r#"
fn branch_before_comment(flag: bool) {
    if flag { return; } // if ignored && ||
}
"#,
        );

        assert_eq!(analysis.function_count, 1);
        assert_eq!(analysis.total_complexity, 2);
        assert_eq!(analysis.max_complexity, 2);
    }

    #[test]
    fn rust_complexity_counts_match_arms() {
        let analysis = analyze_rust_function_complexity(
            r#"
fn classify(x: i32) -> i32 {
    match x {
        0 => 0,
        1 => 1,
        _ => 2,
    }
}
"#,
        );

        assert_eq!(analysis.function_count, 1);
        assert_eq!(analysis.max_complexity, 5);
    }
}
