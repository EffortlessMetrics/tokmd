//! Function-level complexity metrics.
//!
//! This module provides heuristic-based function detection and metrics
//! for common programming languages. It uses regex patterns to identify
//! function definitions and estimates function boundaries using
//! indentation and brace-matching heuristics.
//!
//! ## Supported Languages
//!
//! - Rust: `fn name`
//! - Python: `def name`
//! - JavaScript/TypeScript: `function name`, arrow functions, method syntax
//! - Go: `func name`
//!
//! ## Cyclomatic Complexity
//!
//! This module also provides heuristic-based cyclomatic complexity estimation.
//! It counts decision points per function without full AST parsing:
//!
//! - `if`, `else if`, `elif` -> +1
//! - `match`, `switch`, `case` -> +1 per arm
//! - `for`, `while`, `loop` -> +1
//! - `&&`, `||` (logical operators) -> +1
//! - `?` (ternary/try) -> +1
//! - `catch`, `except` -> +1
//!
//! Base complexity is 1 per function, plus decision points.
//!
//! ## Limitations
//!
//! This is a heuristic approach and may not handle all edge cases:
//! - Nested functions may be double-counted
//! - Multi-line signatures may not be detected correctly
//! - Closures and lambdas have limited support
//! - Keywords in strings/comments may be counted (fast but imperfect)

#![allow(dead_code)]

mod cognitive;
mod cyclomatic;
mod functions;
mod nesting;
mod shared;

#[allow(unused_imports)]
pub use cognitive::{CognitiveComplexity, HighCognitiveFunction, estimate_cognitive_complexity};
#[allow(unused_imports)]
pub use cyclomatic::{
    CyclomaticComplexity, HighComplexityFunction, estimate_cyclomatic_complexity,
};
#[allow(unused_imports)]
pub use functions::{FunctionMetrics, analyze_functions};
// Preserve the historical `content::complexity::NestingAnalysis` path even
// though current callers only use `analyze_nesting_depth` directly.
#[allow(unused_imports)]
pub use nesting::{NestingAnalysis, analyze_nesting_depth};

#[cfg(test)]
#[path = "complexity/tests/unit.rs"]
mod unit_tests;
