//! Function span detection and name extraction helpers.

mod brace;
mod c_style;
mod indentation;
mod javascript;
mod names;
mod patterns;

use brace::detect_brace_functions;
use c_style::detect_c_style_functions;
use indentation::detect_indented_functions;
use javascript::detect_js_functions;
pub(in crate::content::complexity) use names::extract_function_name;
use patterns::{GO_FUNC, PYTHON_DEF, RUST_FN};

/// Detected function with its position and estimated length.
#[derive(Debug, Clone)]
pub(in crate::content::complexity) struct FunctionSpan {
    /// Starting line number (0-indexed).
    pub(in crate::content::complexity) start_line: usize,
    /// Ending line number (0-indexed, inclusive).
    pub(in crate::content::complexity) end_line: usize,
}

impl FunctionSpan {
    #[cfg(test)]
    pub(super) fn length(&self) -> usize {
        self.end_line.saturating_sub(self.start_line) + 1
    }
}

pub(in crate::content::complexity) fn function_spans_for_language(
    lines: &[&str],
    lang: &str,
) -> Vec<FunctionSpan> {
    match lang {
        "rust" | "rs" => detect_brace_functions(lines, &RUST_FN),
        "python" | "py" => detect_indented_functions(lines, &PYTHON_DEF),
        "javascript" | "js" | "typescript" | "ts" | "jsx" | "tsx" => detect_js_functions(lines),
        "go" => detect_brace_functions(lines, &GO_FUNC),
        _ => Vec::new(),
    }
}

pub(in crate::content::complexity) fn function_spans_for_cognitive_language(
    lines: &[&str],
    lang: &str,
) -> Vec<FunctionSpan> {
    match lang {
        "c" | "c++" | "cpp" | "java" | "c#" | "csharp" => detect_c_style_functions(lines),
        _ => function_spans_for_language(lines, lang),
    }
}
