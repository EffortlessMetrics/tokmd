//! Function span detection and length metrics.
//!
//! This module owns language-specific function boundary heuristics shared by
//! function length, cyclomatic complexity, and cognitive complexity analysis.

use regex::Regex;
use std::sync::LazyLock;

use super::shared::get_indent;

/// Metrics about functions in a source file.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionMetrics {
    /// Total number of functions detected.
    pub function_count: usize,
    /// Maximum function length in lines (0 if no functions).
    pub max_function_length: usize,
    /// Average function length in lines (0.0 if no functions).
    pub avg_function_length: f64,
    /// Number of functions exceeding the threshold (default 100 lines).
    pub functions_over_threshold: usize,
}

impl Default for FunctionMetrics {
    fn default() -> Self {
        Self {
            function_count: 0,
            max_function_length: 0,
            avg_function_length: 0.0,
            functions_over_threshold: 0,
        }
    }
}

/// Default threshold for "long" functions.
const LONG_FUNCTION_THRESHOLD: usize = 100;

/// Detected function with its position and estimated length.
#[derive(Debug, Clone)]
pub(super) struct FunctionSpan {
    /// Starting line number (0-indexed).
    pub(super) start_line: usize,
    /// Ending line number (0-indexed, inclusive).
    pub(super) end_line: usize,
}

impl FunctionSpan {
    fn length(&self) -> usize {
        self.end_line.saturating_sub(self.start_line) + 1
    }
}

// Regex patterns for different languages
static RUST_FN: LazyLock<Regex> = LazyLock::new(|| {
    // Qualifiers can appear in various orders: pub async unsafe fn, pub unsafe async fn, etc.
    // Identifier aligns with Rust spec: (XID_Start | _) XID_Continue*
    Regex::new(r#"^\s*(pub(\([^)]+\))?\s+)?((async|unsafe|const|extern\s+"[^"]*")\s+)*fn\s+(?:r#)?(?:_|[\p{XID_Start}])\p{XID_Continue}*"#)
        .expect("Static regex must compile")
});

static PYTHON_DEF: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*(async\s+)?def\s+\w+").expect("Static regex must compile"));

static JS_FUNCTION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*(export\s+)?(async\s+)?function\s+\w+").expect("Static regex must compile")
});

static JS_ARROW: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*(export\s+)?(const|let|var)\s+\w+\s*=\s*(async\s+)?\([^)]*\)\s*=>")
        .expect("Static regex must compile")
});

static JS_METHOD: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*(async\s+)?\w+\s*\([^)]*\)\s*\{").expect("Static regex must compile")
});

static GO_FUNC: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*func\s+\w+").expect("Static regex must compile"));

/// Analyze functions in source code content.
///
/// # Arguments
///
/// * `content` - The source code to analyze.
/// * `language` - The programming language (case-insensitive). Supported:
///   "rust", "python", "javascript", "typescript", "go".
///
/// # Returns
///
/// `FunctionMetrics` containing function count and length statistics.
///
/// # Example
///
/// ```ignore
/// use crate::content::complexity::analyze_functions;
///
/// let rust_code = r#"
/// fn main() {
///     println!("Hello");
/// }
///
/// fn helper() {
///     // one line
/// }
/// "#;
///
/// let metrics = analyze_functions(rust_code, "rust");
/// assert_eq!(metrics.function_count, 2);
/// ```
pub fn analyze_functions(content: &str, language: &str) -> FunctionMetrics {
    let lang = language.to_lowercase();
    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() {
        return FunctionMetrics::default();
    }

    let spans = function_spans_for_language(&lines, &lang);

    compute_metrics(&spans)
}

pub(super) fn function_spans_for_language(lines: &[&str], lang: &str) -> Vec<FunctionSpan> {
    match lang {
        "rust" | "rs" => detect_brace_functions(lines, &RUST_FN),
        "python" | "py" => detect_indented_functions(lines, &PYTHON_DEF),
        "javascript" | "js" | "typescript" | "ts" | "jsx" | "tsx" => detect_js_functions(lines),
        "go" => detect_brace_functions(lines, &GO_FUNC),
        _ => Vec::new(),
    }
}

pub(super) fn function_spans_for_cognitive_language(
    lines: &[&str],
    lang: &str,
) -> Vec<FunctionSpan> {
    match lang {
        "c" | "c++" | "cpp" | "java" | "c#" | "csharp" => detect_c_style_functions(lines),
        _ => function_spans_for_language(lines, lang),
    }
}

/// Extract function name from the line where function starts.
pub(super) fn extract_function_name(lines: &[&str], start_line: usize, lang: &str) -> String {
    let line = lines.get(start_line).unwrap_or(&"");

    match lang {
        "rust" | "rs" => {
            // Look for "fn name" pattern
            if let Some(pos) = line.find("fn ") {
                let after_fn = &line[pos + 3..];
                return extract_identifier(after_fn);
            }
        }
        "python" | "py" => {
            // Look for "def name" pattern
            if let Some(pos) = line.find("def ") {
                let after_def = &line[pos + 4..];
                return extract_identifier(after_def);
            }
        }
        "javascript" | "js" | "typescript" | "ts" | "jsx" | "tsx" => {
            // Look for "function name" pattern
            if let Some(pos) = line.find("function ") {
                let after_func = &line[pos + 9..];
                return extract_identifier(after_func);
            }
            // Look for "const name = " or "let name = " pattern
            if let Some(pos) = line.find("const ") {
                let after_const = &line[pos + 6..];
                return extract_identifier(after_const);
            }
            if let Some(pos) = line.find("let ") {
                let after_let = &line[pos + 4..];
                return extract_identifier(after_let);
            }
            // Method syntax: "name("
            let trimmed = line.trim();
            if let Some(paren_pos) = trimmed.find('(') {
                let before_paren = &trimmed[..paren_pos];
                let words: Vec<&str> = before_paren.split_whitespace().collect();
                if let Some(last) = words.last() {
                    return (*last).to_string();
                }
            }
        }
        "go" => {
            // Look for "func name" pattern
            if let Some(pos) = line.find("func ") {
                let after_func = &line[pos + 5..];
                return extract_identifier(after_func);
            }
        }
        _ => {}
    }

    "unknown".to_string()
}

/// Extract identifier from start of string.
fn extract_identifier(s: &str) -> String {
    let mut name = String::new();
    let mut started = false;

    for ch in s.chars() {
        if !started {
            if ch.is_alphabetic() || ch == '_' {
                started = true;
                name.push(ch);
            }
        } else if ch.is_alphanumeric() || ch == '_' {
            name.push(ch);
        } else {
            break;
        }
    }

    if name.is_empty() {
        "unknown".to_string()
    } else {
        name
    }
}

/// Detect functions in brace-based languages (Rust, Go).
fn detect_brace_functions(lines: &[&str], pattern: &Regex) -> Vec<FunctionSpan> {
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
                // No body found (trait sig, abstract, extern) — skip
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
/// Returns `None` if no opening brace is found (e.g., trait method
/// signatures, extern declarations, abstract methods).
fn find_brace_end(lines: &[&str], start_line: usize) -> Option<usize> {
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

    // Both cases (no open brace, or unclosed braces) -> None
    None
}

/// Detect functions in indentation-based languages (Python).
fn detect_indented_functions(lines: &[&str], pattern: &Regex) -> Vec<FunctionSpan> {
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
                        start = probe; // commit: include this decorator (and skipped blanks)
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

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let indent = get_indent(line);
        if indent <= base_indent {
            // Found a line at same or lower indentation
            return last_content_line;
        }

        last_content_line = i;
    }

    last_content_line
}

/// Detect functions in JavaScript/TypeScript.
fn detect_js_functions(lines: &[&str]) -> Vec<FunctionSpan> {
    let mut spans = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        if JS_FUNCTION.is_match(line) || JS_ARROW.is_match(line) || JS_METHOD.is_match(line) {
            // Avoid matching control structures like if(...) {
            if is_likely_function_start(line) {
                let start = i;
                if let Some(end) = find_brace_end(lines, i) {
                    spans.push(FunctionSpan {
                        start_line: start,
                        end_line: end,
                    });
                    i = end + 1;
                    continue;
                }
            }
        }
        i += 1;
    }

    spans
}

/// Check if a line is likely the start of an actual function (not a method call, etc.).
fn is_likely_function_start(line: &str) -> bool {
    let trimmed = line.trim();
    // Exclude lines that are clearly not function definitions
    !trimmed.starts_with("//")
        && !trimmed.starts_with("/*")
        && !trimmed.starts_with('*')
        && !trimmed.ends_with(',')
        && !trimmed.ends_with(';')
}

/// Detect C-style functions (C, C++, Java, C#).
fn detect_c_style_functions(lines: &[&str]) -> Vec<FunctionSpan> {
    let mut spans = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        // Heuristic: function declaration ends with `) {` or `)` followed by `{` on next line
        let looks_like_fn = trimmed.ends_with(") {")
            || (trimmed.ends_with(')')
                && i + 1 < lines.len()
                && lines[i + 1].trim().starts_with('{'));

        // Exclude control structures
        let is_control = trimmed.starts_with("if ")
            || trimmed.starts_with("if(")
            || trimmed.starts_with("while ")
            || trimmed.starts_with("while(")
            || trimmed.starts_with("for ")
            || trimmed.starts_with("for(")
            || trimmed.starts_with("switch ")
            || trimmed.starts_with("switch(")
            || trimmed.starts_with("catch ")
            || trimmed.starts_with("catch(");

        if looks_like_fn && !is_control {
            let start = i;
            if let Some(end) = find_brace_end(lines, i) {
                spans.push(FunctionSpan {
                    start_line: start,
                    end_line: end,
                });
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

/// Compute metrics from detected function spans.
fn compute_metrics(spans: &[FunctionSpan]) -> FunctionMetrics {
    if spans.is_empty() {
        return FunctionMetrics::default();
    }

    let lengths: Vec<usize> = spans.iter().map(FunctionSpan::length).collect();
    let function_count = lengths.len();
    let max_function_length = lengths.iter().copied().max().unwrap_or(0);
    let total_length: usize = lengths.iter().sum();
    let avg_function_length = total_length as f64 / function_count as f64;
    let functions_over_threshold = lengths
        .iter()
        .filter(|&&len| len > LONG_FUNCTION_THRESHOLD)
        .count();

    FunctionMetrics {
        function_count,
        max_function_length,
        avg_function_length,
        functions_over_threshold,
    }
}
