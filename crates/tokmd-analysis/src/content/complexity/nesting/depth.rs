//! Brace- and indentation-based nesting depth heuristics.

use super::super::shared::{get_indent, is_comment_line};
use super::NestingAnalysis;

/// Analyze brace-based nesting depth.
pub(super) fn analyze_brace_depth(lines: &[&str], lang: &str) -> NestingAnalysis {
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
pub(super) fn analyze_indentation_depth(lines: &[&str]) -> NestingAnalysis {
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
