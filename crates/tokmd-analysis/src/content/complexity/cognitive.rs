//! Cognitive complexity estimation.
//!
//! This module owns the cognitive-complexity result contract and estimator entry
//! point while delegating line-level scoring to owner modules.

mod scoring;

use super::functions;

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
        let cc = scoring::calculate_cognitive_complexity(&func_lines, &lang);
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
