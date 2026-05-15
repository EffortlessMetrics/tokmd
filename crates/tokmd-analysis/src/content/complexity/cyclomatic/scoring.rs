//! Cyclomatic decision-point scoring helpers.

use super::super::shared::{count_keyword, is_comment_line};

/// Calculate cyclomatic complexity for function lines.
pub(super) fn calculate_cyclomatic_complexity(lines: &[&str], lang: &str) -> usize {
    let mut complexity = 1; // Base complexity

    for line in lines {
        let trimmed = line.trim();

        if is_comment_line(trimmed, lang) {
            continue;
        }

        complexity += count_decision_points(trimmed, lang);
    }

    complexity
}

/// Count decision points in a line based on language.
fn count_decision_points(line: &str, lang: &str) -> usize {
    let mut count = 0;

    match lang {
        "rust" | "rs" => {
            let else_if_count = count_keyword(line, "else if ");
            count += else_if_count;
            count += count_standalone_if(line, else_if_count);
            count += count_keyword(line, "match ");
            count += count_keyword(line, "for ");
            count += count_keyword(line, "while ");
            count += count_keyword(line, "loop ");
            count += line.matches("&&").count();
            count += line.matches("||").count();
            count += count_rust_try_op(line);
            count += line.matches("=>").count();
        }
        "python" | "py" => {
            count += count_keyword(line, "if ");
            count += count_keyword(line, "elif ");
            count += count_keyword(line, "for ");
            count += count_keyword(line, "while ");
            count += count_keyword(line, "except ");
            count += count_keyword(line, "except:");
            count += line.matches(" and ").count();
            count += line.matches(" or ").count();
        }
        "javascript" | "js" | "typescript" | "ts" | "jsx" | "tsx" => {
            let else_if_count = count_keyword(line, "else if ") + count_keyword(line, "else if(");
            count += else_if_count;
            count += count_standalone_if_js(line, else_if_count);
            count += count_keyword(line, "switch ");
            count += count_keyword(line, "switch(");
            count += count_keyword(line, "for ");
            count += count_keyword(line, "for(");
            count += count_keyword(line, "while ");
            count += count_keyword(line, "while(");
            count += count_keyword(line, "catch ");
            count += count_keyword(line, "catch(");
            count += count_keyword(line, "case ");
            count += line.matches("&&").count();
            count += line.matches("||").count();
            count += count_ternary_op(line);
        }
        "go" => {
            let else_if_count = count_keyword(line, "else if ");
            count += else_if_count;
            count += count_standalone_if(line, else_if_count);
            count += count_keyword(line, "switch ");
            count += count_keyword(line, "select ");
            count += count_keyword(line, "for ");
            count += count_keyword(line, "case ");
            count += line.matches("&&").count();
            count += line.matches("||").count();
        }
        _ => {}
    }

    count
}

/// Count standalone `if ` occurrences, excluding those that are part of `else if `.
fn count_standalone_if(line: &str, else_if_count: usize) -> usize {
    let total_if = count_keyword(line, "if ");
    total_if.saturating_sub(else_if_count)
}

/// Count standalone `if` occurrences in JS, handling both `if ` and `if(`.
fn count_standalone_if_js(line: &str, else_if_count: usize) -> usize {
    let total_if = count_keyword(line, "if ") + count_keyword(line, "if(");
    total_if.saturating_sub(else_if_count)
}

/// Count Rust try operator `?` at expression end.
fn count_rust_try_op(line: &str) -> usize {
    let mut count = 0;
    let chars: Vec<char> = line.chars().collect();

    for (i, &ch) in chars.iter().enumerate() {
        if ch == '?' {
            let prev = if i > 0 { chars.get(i - 1) } else { None };
            let next = chars.get(i + 1);

            if prev == Some(&':') || prev == Some(&'#') {
                continue;
            }

            let is_try = next.is_none()
                || matches!(
                    next,
                    Some(';') | Some(')') | Some('}') | Some(',') | Some(' ')
                );
            let is_optional_chain = next == Some(&'.');

            if is_try && !is_optional_chain {
                count += 1;
            }
        }
    }

    count
}

/// Count ternary operators in JS/TS.
fn count_ternary_op(line: &str) -> usize {
    let mut count = 0;
    let chars: Vec<char> = line.chars().collect();

    for (i, &ch) in chars.iter().enumerate() {
        if ch == '?' {
            let next = chars.get(i + 1);
            let is_optional_chain = next == Some(&'.');
            let at_end = next.is_none() || matches!(next, Some(';') | Some(')'));
            let has_colon = chars[i..].contains(&':');

            if !is_optional_chain && !at_end && has_colon {
                count += 1;
            }
        }
    }

    count
}
