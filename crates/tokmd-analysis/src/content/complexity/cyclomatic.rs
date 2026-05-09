//! Cyclomatic complexity estimation.
//!
//! This module owns decision-point scoring while sharing function span
//! detection and low-level source predicates with the parent complexity module.

use super::{count_keyword, functions, is_comment_line};

/// Result of cyclomatic complexity analysis.
#[derive(Debug, Clone, PartialEq)]
pub struct CyclomaticComplexity {
    /// Sum of complexity across all detected functions.
    pub total_cc: usize,
    /// Maximum complexity of any single function.
    pub max_cc: usize,
    /// Average complexity per function.
    pub avg_cc: f64,
    /// Functions with complexity > 10 (considered high complexity).
    pub high_complexity_functions: Vec<HighComplexityFunction>,
    /// Number of functions detected.
    pub function_count: usize,
}

/// A function identified as having high cyclomatic complexity (CC > 10).
#[derive(Debug, Clone, PartialEq)]
pub struct HighComplexityFunction {
    /// Approximate name or identifier of the function.
    pub name: String,
    /// Line number where the function starts (1-indexed).
    pub line: usize,
    /// Cyclomatic complexity value.
    pub complexity: usize,
}

impl Default for CyclomaticComplexity {
    fn default() -> Self {
        Self {
            total_cc: 0,
            max_cc: 0,
            avg_cc: 0.0,
            high_complexity_functions: Vec::new(),
            function_count: 0,
        }
    }
}

/// Threshold for high complexity functions.
const HIGH_COMPLEXITY_THRESHOLD: usize = 10;

/// Estimate cyclomatic complexity of code content using pattern matching.
///
/// This is a heuristic approach that:
/// 1. Identifies functions via pattern matching
/// 2. Counts decision points within each function
/// 3. Calculates CC = 1 + decision_points for each function
///
/// # Arguments
/// * `content` - Source code as a string
/// * `language` - Language name (case-insensitive): "rust", "python", "javascript", etc.
///
/// # Returns
/// Cyclomatic complexity analysis results. Returns default (empty) results for
/// unsupported languages.
///
/// # Example
/// ```ignore
/// use crate::content::complexity::estimate_cyclomatic_complexity;
///
/// let rust_code = r#"
/// fn simple() {
///     println!("hello");
/// }
///
/// fn complex(x: i32) -> i32 {
///     if x > 0 {
///         if x > 10 {
///             return x * 2;
///         }
///         return x;
///     } else {
///         return 0;
///     }
/// }
/// "#;
///
/// let result = estimate_cyclomatic_complexity(rust_code, "rust");
/// assert_eq!(result.function_count, 2);
/// assert!(result.max_cc >= 2); // complex() has at least 2 decision points
/// ```
pub fn estimate_cyclomatic_complexity(content: &str, language: &str) -> CyclomaticComplexity {
    let lang = language.to_lowercase();
    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() {
        return CyclomaticComplexity::default();
    }

    // Get function spans using shared language detection.
    let spans = functions::function_spans_for_language(&lines, &lang);

    if spans.is_empty() {
        return CyclomaticComplexity::default();
    }

    let mut complexities: Vec<(String, usize, usize)> = Vec::new(); // (name, line, cc)

    for span in &spans {
        let func_name = functions::extract_function_name(&lines, span.start_line, &lang);
        let func_lines: Vec<&str> = lines[span.start_line..=span.end_line].to_vec();
        let cc = calculate_cyclomatic_complexity(&func_lines, &lang);
        complexities.push((func_name, span.start_line + 1, cc)); // 1-indexed line
    }

    let total_cc: usize = complexities.iter().map(|(_, _, cc)| cc).sum();
    let max_cc = complexities.iter().map(|(_, _, cc)| *cc).max().unwrap_or(0);
    let function_count = complexities.len();
    let avg_cc = if function_count > 0 {
        total_cc as f64 / function_count as f64
    } else {
        0.0
    };

    let high_complexity_functions: Vec<HighComplexityFunction> = complexities
        .iter()
        .filter(|(_, _, cc)| *cc > HIGH_COMPLEXITY_THRESHOLD)
        .map(|(name, line, cc)| HighComplexityFunction {
            name: name.clone(),
            line: *line,
            complexity: *cc,
        })
        .collect();

    CyclomaticComplexity {
        total_cc,
        max_cc,
        avg_cc,
        high_complexity_functions,
        function_count,
    }
}

/// Calculate cyclomatic complexity for function lines.
fn calculate_cyclomatic_complexity(lines: &[&str], lang: &str) -> usize {
    let mut complexity = 1; // Base complexity

    for line in lines {
        let trimmed = line.trim();

        // Skip comments
        if is_comment_line(trimmed, lang) {
            continue;
        }

        // Count decision points based on language
        complexity += count_decision_points(trimmed, lang);
    }

    complexity
}

/// Count decision points in a line based on language.
fn count_decision_points(line: &str, lang: &str) -> usize {
    let mut count = 0;

    match lang {
        "rust" | "rs" => {
            // Count else if first, then standalone if (avoiding double-count)
            let else_if_count = count_keyword(line, "else if ");
            count += else_if_count;
            count += count_standalone_if(line, else_if_count);
            count += count_keyword(line, "match ");
            count += count_keyword(line, "for ");
            count += count_keyword(line, "while ");
            count += count_keyword(line, "loop ");
            count += line.matches("&&").count();
            count += line.matches("||").count();
            count += count_rust_try_op(line);
            count += line.matches("=>").count(); // Match arms
        }
        "python" | "py" => {
            count += count_keyword(line, "if ");
            count += count_keyword(line, "elif ");
            count += count_keyword(line, "for ");
            count += count_keyword(line, "while ");
            count += count_keyword(line, "except ");
            count += count_keyword(line, "except:");
            count += line.matches(" and ").count();
            count += line.matches(" or ").count();
        }
        "javascript" | "js" | "typescript" | "ts" | "jsx" | "tsx" => {
            // Count else if first, then standalone if (avoiding double-count)
            let else_if_count = count_keyword(line, "else if ") + count_keyword(line, "else if(");
            count += else_if_count;
            count += count_standalone_if_js(line, else_if_count);
            count += count_keyword(line, "switch ");
            count += count_keyword(line, "switch(");
            count += count_keyword(line, "for ");
            count += count_keyword(line, "for(");
            count += count_keyword(line, "while ");
            count += count_keyword(line, "while(");
            count += count_keyword(line, "catch ");
            count += count_keyword(line, "catch(");
            count += count_keyword(line, "case ");
            count += line.matches("&&").count();
            count += line.matches("||").count();
            count += count_ternary_op(line);
        }
        "go" => {
            // Count else if first, then standalone if (avoiding double-count)
            let else_if_count = count_keyword(line, "else if ");
            count += else_if_count;
            count += count_standalone_if(line, else_if_count);
            count += count_keyword(line, "switch ");
            count += count_keyword(line, "select ");
            count += count_keyword(line, "for ");
            count += count_keyword(line, "case ");
            count += line.matches("&&").count();
            count += line.matches("||").count();
        }
        _ => {}
    }

    count
}

/// Count standalone `if ` occurrences, excluding those that are part of `else if `.
fn count_standalone_if(line: &str, else_if_count: usize) -> usize {
    let total_if = count_keyword(line, "if ");
    // Subtract the if's that are part of else if
    total_if.saturating_sub(else_if_count)
}

/// Count standalone `if` occurrences in JS (handles both `if ` and `if(`).
fn count_standalone_if_js(line: &str, else_if_count: usize) -> usize {
    let total_if = count_keyword(line, "if ") + count_keyword(line, "if(");
    // Subtract the if's that are part of else if
    total_if.saturating_sub(else_if_count)
}

/// Count Rust try operator `?` (at expression end).
fn count_rust_try_op(line: &str) -> usize {
    let mut count = 0;
    let chars: Vec<char> = line.chars().collect();

    for (i, &ch) in chars.iter().enumerate() {
        if ch == '?' {
            let prev = if i > 0 { chars.get(i - 1) } else { None };
            let next = chars.get(i + 1);

            // Exclude format specifiers like {:?} or {:#?}
            // These have ? preceded by : or #
            if prev == Some(&':') || prev == Some(&'#') {
                continue;
            }

            // Try operator is ? followed by end-of-expression characters
            let is_try = next.is_none()
                || matches!(
                    next,
                    Some(';') | Some(')') | Some('}') | Some(',') | Some(' ')
                );
            // Exclude ?. (optional chaining)
            let is_optional_chain = next == Some(&'.');

            if is_try && !is_optional_chain {
                count += 1;
            }
        }
    }

    count
}

/// Count ternary operators in JS/TS.
fn count_ternary_op(line: &str) -> usize {
    let mut count = 0;
    let chars: Vec<char> = line.chars().collect();

    for (i, &ch) in chars.iter().enumerate() {
        if ch == '?' {
            let next = chars.get(i + 1);
            // Ternary: ? not followed by . (optional chain) or at end
            let is_optional_chain = next == Some(&'.');
            let at_end = next.is_none() || matches!(next, Some(';') | Some(')'));
            // Check there's a : somewhere after
            let has_colon = line[i..].contains(':');

            if !is_optional_chain && !at_end && has_colon {
                count += 1;
            }
        }
    }

    count
}
