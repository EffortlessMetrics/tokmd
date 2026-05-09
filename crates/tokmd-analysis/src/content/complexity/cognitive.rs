//! Cognitive complexity estimation.
//!
//! This module owns nesting-aware cognitive scoring while sharing function span
//! detection and low-level source predicates with the parent complexity module.

use super::{count_keyword, functions, is_comment_line};

/// Result of cognitive complexity analysis.
///
/// Cognitive complexity differs from cyclomatic complexity by penalizing
/// nested control structures more heavily. Each level of nesting adds
/// an additional increment to the complexity score.
#[derive(Debug, Clone, PartialEq)]
pub struct CognitiveComplexity {
    /// Sum of cognitive complexity across all detected functions.
    pub total: usize,
    /// Maximum cognitive complexity of any single function.
    pub max: usize,
    /// Average cognitive complexity per function.
    pub avg: f64,
    /// Number of functions detected.
    pub function_count: usize,
    /// Functions with cognitive complexity > threshold (default 15).
    pub high_complexity_functions: Vec<HighCognitiveFunction>,
}

/// A function identified as having high cognitive complexity.
#[derive(Debug, Clone, PartialEq)]
pub struct HighCognitiveFunction {
    /// Approximate name or identifier of the function.
    pub name: String,
    /// Line number where the function starts (1-indexed).
    pub line: usize,
    /// Cognitive complexity value.
    pub complexity: usize,
}

impl Default for CognitiveComplexity {
    fn default() -> Self {
        Self {
            total: 0,
            max: 0,
            avg: 0.0,
            function_count: 0,
            high_complexity_functions: Vec::new(),
        }
    }
}

/// Threshold for high cognitive complexity functions.
const HIGH_COGNITIVE_THRESHOLD: usize = 15;

/// Estimate cognitive complexity of code content using pattern matching.
///
/// Cognitive complexity scoring:
/// - Control structures (if, for, while, etc.): +1 + nesting_level
/// - Logical operator sequences (&&, ||): +1 per sequence
/// - Break/continue with labels: +1
/// - Recursion: +1 (not currently detected)
///
/// # Arguments
/// * `content` - Source code as a string
/// * `language` - Language name (case-insensitive): "rust", "python", "javascript", etc.
///
/// # Returns
/// Cognitive complexity analysis results.
///
/// # Example
/// ```ignore
/// use crate::content::complexity::estimate_cognitive_complexity;
///
/// let rust_code = r#"
/// fn complex(x: i32) -> i32 {
///     if x > 0 {
///         if x > 10 {
///             return x * 2;
///         }
///     }
///     0
/// }
/// "#;
///
/// let result = estimate_cognitive_complexity(rust_code, "rust");
/// assert_eq!(result.function_count, 1);
/// assert!(result.max >= 3); // Nested if adds more cognitive load
/// ```
pub fn estimate_cognitive_complexity(content: &str, language: &str) -> CognitiveComplexity {
    let lang = language.to_lowercase();
    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() {
        return CognitiveComplexity::default();
    }

    // Get function spans using shared language detection.
    let spans = functions::function_spans_for_cognitive_language(&lines, &lang);

    if spans.is_empty() {
        return CognitiveComplexity::default();
    }

    let mut complexities: Vec<(String, usize, usize)> = Vec::new(); // (name, line, cc)

    for span in &spans {
        let func_name = functions::extract_function_name(&lines, span.start_line, &lang);
        let func_lines: Vec<&str> = lines[span.start_line..=span.end_line].to_vec();
        let cc = calculate_cognitive_complexity(&func_lines, &lang);
        complexities.push((func_name, span.start_line + 1, cc)); // 1-indexed line
    }

    let total: usize = complexities.iter().map(|(_, _, cc)| cc).sum();
    let max = complexities.iter().map(|(_, _, cc)| *cc).max().unwrap_or(0);
    let function_count = complexities.len();
    let avg = if function_count > 0 {
        total as f64 / function_count as f64
    } else {
        0.0
    };

    let high_complexity_functions: Vec<HighCognitiveFunction> = complexities
        .iter()
        .filter(|(_, _, cc)| *cc > HIGH_COGNITIVE_THRESHOLD)
        .map(|(name, line, cc)| HighCognitiveFunction {
            name: name.clone(),
            line: *line,
            complexity: *cc,
        })
        .collect();

    CognitiveComplexity {
        total,
        max,
        avg,
        function_count,
        high_complexity_functions,
    }
}

/// Calculate cognitive complexity for function lines.
fn calculate_cognitive_complexity(lines: &[&str], lang: &str) -> usize {
    let mut complexity = 0usize;
    let mut nesting_depth = 0usize;
    let mut in_logical_sequence = false;

    for line in lines {
        let trimmed = line.trim();

        // Skip comments
        if is_comment_line(trimmed, lang) {
            continue;
        }

        // Track nesting for brace-based languages
        let opens = count_structure_opens(trimmed, lang);
        let closes = count_structure_closes(trimmed, lang);

        // Add complexity for control structures with nesting penalty
        let control_structures = count_control_structures(trimmed, lang);
        for _ in 0..control_structures {
            complexity += 1 + nesting_depth;
        }

        // Add complexity for logical operator sequences
        let (new_in_sequence, seq_complexity) =
            count_logical_sequences(trimmed, in_logical_sequence);
        complexity += seq_complexity;
        in_logical_sequence = new_in_sequence;

        // Add complexity for break/continue with labels (Rust-specific)
        if lang == "rust" || lang == "rs" {
            complexity += count_labeled_jumps(trimmed);
        }

        // Update nesting depth
        nesting_depth = nesting_depth.saturating_add(opens);
        nesting_depth = nesting_depth.saturating_sub(closes);
    }

    complexity
}

/// Count control structure keywords that add to cognitive complexity.
fn count_control_structures(line: &str, lang: &str) -> usize {
    let mut count = 0;

    match lang {
        "rust" | "rs" => {
            // Count standalone if (not else if, which is already counted as one)
            if line.contains("if ") && !line.contains("else if ") {
                count += line.matches("if ").count();
            }
            if line.contains("else if ") {
                count += line.matches("else if ").count();
            }
            count += count_keyword(line, "match ");
            count += count_keyword(line, "for ");
            count += count_keyword(line, "while ");
            count += count_keyword(line, "loop ");
        }
        "python" | "py" => {
            count += count_keyword(line, "if ");
            count += count_keyword(line, "elif ");
            count += count_keyword(line, "for ");
            count += count_keyword(line, "while ");
            count += count_keyword(line, "except ");
            count += count_keyword(line, "except:");
        }
        "javascript" | "js" | "typescript" | "ts" | "jsx" | "tsx" => {
            // Count if statements (avoid double-counting else if)
            let else_if_count = count_keyword(line, "else if ") + count_keyword(line, "else if(");
            count += else_if_count;
            let total_if = count_keyword(line, "if ") + count_keyword(line, "if(");
            count += total_if.saturating_sub(else_if_count);
            count += count_keyword(line, "switch ");
            count += count_keyword(line, "switch(");
            count += count_keyword(line, "for ");
            count += count_keyword(line, "for(");
            count += count_keyword(line, "while ");
            count += count_keyword(line, "while(");
            count += count_keyword(line, "catch ");
            count += count_keyword(line, "catch(");
        }
        "go" => {
            let else_if_count = count_keyword(line, "else if ");
            count += else_if_count;
            let total_if = count_keyword(line, "if ");
            count += total_if.saturating_sub(else_if_count);
            count += count_keyword(line, "switch ");
            count += count_keyword(line, "select ");
            count += count_keyword(line, "for ");
        }
        "c" | "c++" | "cpp" | "java" | "c#" | "csharp" => {
            let else_if_count = count_keyword(line, "else if ") + count_keyword(line, "else if(");
            count += else_if_count;
            let total_if = count_keyword(line, "if ") + count_keyword(line, "if(");
            count += total_if.saturating_sub(else_if_count);
            count += count_keyword(line, "switch ");
            count += count_keyword(line, "switch(");
            count += count_keyword(line, "for ");
            count += count_keyword(line, "for(");
            count += count_keyword(line, "while ");
            count += count_keyword(line, "while(");
            count += count_keyword(line, "catch ");
            count += count_keyword(line, "catch(");
        }
        _ => {}
    }

    count
}

/// Count structure-opening keywords/braces.
fn count_structure_opens(line: &str, lang: &str) -> usize {
    match lang {
        "python" | "py" => {
            // Python uses indentation, not braces, so we count structure keywords
            let mut count = 0;
            if line.contains("if ") || line.contains("elif ") {
                count += 1;
            }
            if line.contains("for ") || line.contains("while ") {
                count += 1;
            }
            if line.contains("try:") || line.contains("except ") || line.contains("except:") {
                count += 1;
            }
            if line.contains("with ") {
                count += 1;
            }
            count
        }
        _ => line.chars().filter(|&c| c == '{').count(),
    }
}

/// Count structure-closing keywords/braces.
fn count_structure_closes(line: &str, lang: &str) -> usize {
    match lang {
        "python" | "py" => {
            // For Python, closing is determined by dedent, which is harder to detect
            // We use a simplified heuristic: count pass/return/break/continue
            0
        }
        _ => line.chars().filter(|&c| c == '}').count(),
    }
}

/// Count logical operator sequences that add to cognitive complexity.
/// Returns (still_in_sequence, complexity_added).
fn count_logical_sequences(line: &str, was_in_sequence: bool) -> (bool, usize) {
    let has_and = line.contains("&&") || line.contains(" and ");
    let has_or = line.contains("||") || line.contains(" or ");

    if has_and || has_or {
        // If we weren't in a sequence, starting one adds 1
        // If we are continuing, no additional cost
        let cost = if was_in_sequence { 0 } else { 1 };
        (true, cost)
    } else {
        (false, 0)
    }
}

/// Count labeled break/continue statements in Rust.
fn count_labeled_jumps(line: &str) -> usize {
    // Look for patterns like `break 'label` or `continue 'label`
    let mut count = 0;

    // Simple pattern: break/continue followed by a tick (label)
    if line.contains("break '") {
        count += 1;
    }
    if line.contains("continue '") {
        count += 1;
    }

    count
}
