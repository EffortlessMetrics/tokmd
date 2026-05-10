//! Nesting-depth analysis for source complexity.
//!
//! This module owns the nesting-depth result contract and entry point used by
//! the content complexity analyzer.

mod depth;

/// Result of nesting depth analysis.
#[derive(Debug, Clone, PartialEq)]
pub struct NestingAnalysis {
    /// Maximum nesting depth found in the code.
    pub max_depth: usize,
    /// Average nesting depth across the code.
    pub avg_depth: f64,
    /// Line numbers where maximum nesting depth was reached (1-indexed).
    pub max_depth_lines: Vec<usize>,
}

impl Default for NestingAnalysis {
    fn default() -> Self {
        Self {
            max_depth: 0,
            avg_depth: 0.0,
            max_depth_lines: Vec::new(),
        }
    }
}

/// Analyze nesting depth in source code.
///
/// For brace-based languages (Rust, C, JS, Go, etc.), tracks brace depth.
/// For Python, tracks indentation level.
///
/// # Arguments
/// * `content` - Source code as a string
/// * `language` - Language name (case-insensitive)
///
/// # Returns
/// `NestingAnalysis` with max depth, average depth, and line numbers of max depth.
///
/// # Example
/// ```ignore
/// use crate::content::complexity::analyze_nesting_depth;
///
/// let rust_code = r#"
/// fn main() {
///     if true {
///         for i in 0..10 {
///             println!("{}", i);
///         }
///     }
/// }
/// "#;
///
/// let result = analyze_nesting_depth(rust_code, "rust");
/// // Depth: fn=1, if=2, for=3, inside for body=4 when processing the for line
/// assert!(result.max_depth >= 3);
/// ```
pub fn analyze_nesting_depth(content: &str, language: &str) -> NestingAnalysis {
    let lang = language.to_lowercase();
    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() {
        return NestingAnalysis::default();
    }

    match lang.as_str() {
        "python" | "py" => depth::analyze_indentation_depth(&lines),
        _ => depth::analyze_brace_depth(&lines, &lang),
    }
}
