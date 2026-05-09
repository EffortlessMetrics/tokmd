//! Function span detection and length metrics.
//!
//! This module owns language-specific function boundary heuristics shared by
//! function length, cyclomatic complexity, and cognitive complexity analysis.

mod spans;

pub(super) use spans::{
    extract_function_name, function_spans_for_cognitive_language, function_spans_for_language,
};

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

/// Compute metrics from detected function spans.
fn compute_metrics(spans: &[spans::FunctionSpan]) -> FunctionMetrics {
    if spans.is_empty() {
        return FunctionMetrics::default();
    }

    let lengths: Vec<usize> = spans.iter().map(spans::FunctionSpan::length).collect();
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
