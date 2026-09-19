//! Source token counting for Halstead analysis.

use std::collections::{BTreeMap, BTreeSet};

use super::operators::operators_for_lang;

/// Per-file Halstead token counts.
pub(crate) struct FileTokenCounts {
    pub operators: BTreeMap<String, usize>,
    pub operands: BTreeSet<String>,
    pub total_operators: usize,
    pub total_operands: usize,
}

/// Tokenize source code into operators and operands for Halstead analysis.
pub(crate) fn tokenize_for_halstead(text: &str, lang: &str) -> FileTokenCounts {
    let ops = operators_for_lang(lang);
    let op_set: BTreeSet<&str> = ops.iter().copied().collect();

    let mut operators: BTreeMap<String, usize> = BTreeMap::new();
    let mut operands: BTreeSet<String> = BTreeSet::new();
    let mut total_operators = 0usize;
    let mut total_operands = 0usize;

    for line in text.lines() {
        let trimmed = line.trim();
        // Skip comments and empty lines.
        if trimmed.is_empty()
            || trimmed.starts_with("//")
            || trimmed.starts_with('#')
            || trimmed.starts_with("/*")
            || trimmed.starts_with('*')
        {
            continue;
        }

        let mut chars = trimmed.chars().peekable();
        while let Some(&ch) = chars.peek() {
            if ch.is_whitespace() {
                chars.next();
                continue;
            }

            if ch.is_ascii_punctuation() && ch != '_' && ch != '"' && ch != '\'' {
                let remaining: String = chars.clone().take(4).collect();
                let mut matched = false;
                let char_count = remaining.chars().count();
                for len in (1..=char_count.min(4)).rev() {
                    let (byte_idx, _) = remaining
                        .char_indices()
                        .nth(len)
                        .unwrap_or((remaining.len(), '\0'));
                    let candidate = &remaining[..byte_idx];
                    if op_set.contains(candidate) {
                        *operators.entry(candidate.to_string()).or_insert(0) += 1;
                        total_operators += 1;
                        for _ in 0..len {
                            chars.next();
                        }
                        matched = true;
                        break;
                    }
                }
                if !matched {
                    chars.next();
                }
                continue;
            }

            if ch.is_alphanumeric() || ch == '_' {
                let mut word = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_alphanumeric() || c == '_' {
                        word.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if op_set.contains(word.as_str()) {
                    *operators.entry(word).or_insert(0) += 1;
                    total_operators += 1;
                } else {
                    operands.insert(word);
                    total_operands += 1;
                }
                continue;
            }

            if ch == '"' || ch == '\'' {
                chars.next();
                let quote = ch;
                while let Some(&c) = chars.peek() {
                    chars.next();
                    if c == '\\' {
                        chars.next();
                    } else if c == quote {
                        break;
                    }
                }
                total_operands += 1;
                operands.insert("<string>".to_string());
                continue;
            }

            chars.next();
        }
    }

    FileTokenCounts {
        operators,
        operands,
        total_operators,
        total_operands,
    }
}
