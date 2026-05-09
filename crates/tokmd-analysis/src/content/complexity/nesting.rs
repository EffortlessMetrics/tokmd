//! Nesting-depth analysis for source complexity.
//!
//! This module owns brace- and indentation-based nesting depth heuristics used
//! by the content complexity analyzer.

use super::{shared::get_indent, shared::is_comment_line};

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
        "python" | "py" => analyze_indentation_depth(&lines),
        _ => analyze_brace_depth(&lines, &lang),
    }
}

/// Analyze brace-based nesting depth.
fn analyze_brace_depth(lines: &[&str], lang: &str) -> NestingAnalysis {
    let mut current_depth = 0usize;
    let mut max_depth = 0usize;
    let mut max_depth_lines: Vec<usize> = Vec::new();
    let mut total_depth = 0usize;
    let mut counted_lines = 0usize;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Skip comments
        if is_comment_line(trimmed, lang) {
            continue;
        }

        // Count braces
        let opens = line.chars().filter(|&c| c == '{').count();
        let closes = line.chars().filter(|&c| c == '}').count();

        // Update depth based on order of braces in line
        // If line has both, the depth between them may be higher
        let line_max_depth = current_depth + opens;

        if line_max_depth > max_depth {
            max_depth = line_max_depth;
            max_depth_lines.clear();
            max_depth_lines.push(i + 1); // 1-indexed
        } else if line_max_depth == max_depth && !max_depth_lines.contains(&(i + 1)) {
            max_depth_lines.push(i + 1);
        }

        current_depth = current_depth.saturating_add(opens);
        current_depth = current_depth.saturating_sub(closes);

        total_depth += current_depth;
        counted_lines += 1;
    }

    let avg_depth = if counted_lines > 0 {
        total_depth as f64 / counted_lines as f64
    } else {
        0.0
    };

    NestingAnalysis {
        max_depth,
        avg_depth,
        max_depth_lines,
    }
}

/// Analyze indentation-based nesting depth (Python).
fn analyze_indentation_depth(lines: &[&str]) -> NestingAnalysis {
    let mut max_depth = 0usize;
    let mut max_depth_lines: Vec<usize> = Vec::new();
    let mut total_depth = 0usize;
    let mut counted_lines = 0usize;

    // Detect indentation unit (2 or 4 spaces, or tab)
    let indent_unit = detect_indent_unit(lines);

    for (i, line) in lines.iter().enumerate() {
        if line.trim().is_empty() || line.trim().starts_with('#') {
            continue;
        }

        let indent = get_indent(line);
        let depth = indent.checked_div(indent_unit).unwrap_or(0);

        if depth > max_depth {
            max_depth = depth;
            max_depth_lines.clear();
            max_depth_lines.push(i + 1);
        } else if depth == max_depth && !max_depth_lines.contains(&(i + 1)) {
            max_depth_lines.push(i + 1);
        }

        total_depth += depth;
        counted_lines += 1;
    }

    let avg_depth = if counted_lines > 0 {
        total_depth as f64 / counted_lines as f64
    } else {
        0.0
    };

    NestingAnalysis {
        max_depth,
        avg_depth,
        max_depth_lines,
    }
}

/// Detect the indentation unit used in Python code.
fn detect_indent_unit(lines: &[&str]) -> usize {
    for line in lines {
        if line.starts_with('\t') {
            // Tab-based indentation; treat as 1 unit
            return 1;
        }
        let indent = get_indent(line);
        if indent > 0 && indent <= 8 {
            return indent;
        }
    }
    4 // Default to 4 spaces
}
