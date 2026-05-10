//! Line-level cognitive-complexity scoring.
//!
//! This module owns the heuristic control-flow, nesting, logical-sequence, and
//! labeled-jump scoring rules used by the cognitive complexity estimator.

use super::super::shared::{count_keyword, is_comment_line};

/// Calculate cognitive complexity for function lines.
pub(super) fn calculate_cognitive_complexity(lines: &[&str], lang: &str) -> usize {
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
