use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::Result;
use tokmd_analysis_types::HalsteadMetrics;
use tokmd_types::{ExportData, FileKind, FileRow};

use crate::analysis::AnalysisLimits;
use crate::util::normalize_path;

const DEFAULT_MAX_FILE_BYTES: u64 = 128 * 1024;

/// Languages that support Halstead analysis.
fn is_halstead_lang(lang: &str) -> bool {
    matches!(
        lang.to_lowercase().as_str(),
        "rust"
            | "javascript"
            | "typescript"
            | "python"
            | "go"
            | "c"
            | "c++"
            | "java"
            | "c#"
            | "php"
            | "ruby"
    )
}

/// Operators for a given language.
fn operators_for_lang(lang: &str) -> &'static [&'static str] {
    match lang.to_lowercase().as_str() {
        "rust" => &[
            "fn", "let", "mut", "if", "else", "match", "while", "for", "loop", "return", "break",
            "continue", "struct", "enum", "impl", "trait", "pub", "use", "mod", "async", "await",
            "move", "ref", "self", "Self", "where", "type", "const", "static", "unsafe", "extern",
            "crate", "super", "as", "in", "->", "=>", "::", "=", "==", "!=", "<", ">", "<=",
            ">=", "+", "-", "*", "/", "%", "&", "|", "^", "!", "&&", "||", "<<", ">>", "+=",
            "-=", "*=", "/=", "%=", "&=", "|=", "^=", "<<=", ">>=", "..", "..=", "?",
        ],
        "javascript" | "typescript" => &[
            "function", "var", "let", "const", "if", "else", "switch", "case", "while", "for",
            "do", "return", "break", "continue", "class", "new", "this", "super", "import",
            "export", "default", "try", "catch", "finally", "throw", "async", "await", "yield",
            "typeof", "instanceof", "in", "of", "delete", "void", "=>", "=", "==", "===", "!=",
            "!==", "<", ">", "<=", ">=", "+", "-", "*", "/", "%", "&", "|", "^", "!", "&&",
            "||", "~", "<<", ">>", ">>>", "+=", "-=", "*=", "/=", "%=", "&=", "|=", "^=",
            "<<=", ">>=", ">>>=", "**", "**=", "?.", "??", "??=", "++", "--", "...",
        ],
        "python" => &[
            "def", "class", "if", "elif", "else", "while", "for", "return", "break", "continue",
            "import", "from", "as", "try", "except", "finally", "raise", "with", "yield",
            "lambda", "pass", "del", "global", "nonlocal", "assert", "async", "await", "and",
            "or", "not", "is", "in", "=", "==", "!=", "<", ">", "<=", ">=", "+", "-", "*", "/",
            "//", "%", "**", "&", "|", "^", "~", "<<", ">>", "+=", "-=", "*=", "/=", "//=",
            "%=", "**=", "&=", "|=", "^=", "<<=", ">>=", ":=",
        ],
        "go" => &[
            "func", "var", "const", "type", "struct", "interface", "if", "else", "switch", "case",
            "for", "range", "return", "break", "continue", "goto", "defer", "go", "select",
            "chan", "map", "package", "import", "fallthrough", ":=", "=", "==", "!=", "<", ">",
            "<=", ">=", "+", "-", "*", "/", "%", "&", "|", "^", "!", "&&", "||", "<<", ">>",
            "&^", "+=", "-=", "*=", "/=", "%=", "&=", "|=", "^=", "<<=", ">>=", "&^=", "++",
            "--", "<-",
        ],
        "c" | "c++" | "java" | "c#" | "php" => &[
            "if", "else", "switch", "case", "while", "for", "do", "return", "break", "continue",
            "class", "struct", "new", "delete", "try", "catch", "throw", "finally", "void",
            "static", "const", "public", "private", "protected", "virtual", "override",
            "abstract", "interface", "extends", "implements", "import", "include", "using",
            "namespace", "typedef", "template", "=", "==", "!=", "<", ">", "<=", ">=", "+", "-",
            "*", "/", "%", "&", "|", "^", "!", "&&", "||", "~", "<<", ">>", "+=", "-=", "*=",
            "/=", "%=", "&=", "|=", "^=", "<<=", ">>=", "++", "--", "->", "::", "sizeof",
            "typeof",
        ],
        "ruby" => &[
            "def", "class", "module", "if", "elsif", "else", "unless", "while", "until", "for",
            "do", "return", "break", "next", "begin", "rescue", "ensure", "raise", "yield",
            "require", "include", "extend", "attr_reader", "attr_writer", "attr_accessor", "self",
            "super", "end", "and", "or", "not", "=", "==", "!=", "<", ">", "<=", ">=", "+", "-",
            "*", "/", "%", "&", "|", "^", "!", "&&", "||", "~", "<<", ">>", "+=", "-=", "*=",
            "/=", "%=", "&=", "|=", "^=", "<<=", ">>=", "**", "**=", "<=>", "=~", "!~", "..",
            "...",
        ],
        _ => &[],
    }
}

/// Per-file Halstead token counts.
struct FileTokenCounts {
    operators: BTreeMap<String, usize>,
    operands: BTreeSet<String>,
    total_operators: usize,
    total_operands: usize,
}

/// Tokenize source code into operators and operands for Halstead analysis.
fn tokenize_for_halstead(text: &str, lang: &str) -> FileTokenCounts {
    let ops = operators_for_lang(lang);
    let op_set: BTreeSet<&str> = ops.iter().copied().collect();

    let mut operators: BTreeMap<String, usize> = BTreeMap::new();
    let mut operands: BTreeSet<String> = BTreeSet::new();
    let mut total_operators = 0usize;
    let mut total_operands = 0usize;

    for line in text.lines() {
        let trimmed = line.trim();
        // Skip comments and empty lines
        if trimmed.is_empty()
            || trimmed.starts_with("//")
            || trimmed.starts_with('#')
            || trimmed.starts_with("/*")
            || trimmed.starts_with('*')
        {
            continue;
        }

        // Extract words and symbols
        let mut chars = trimmed.chars().peekable();
        while let Some(&ch) = chars.peek() {
            if ch.is_whitespace() {
                chars.next();
                continue;
            }

            // Try to match multi-char operators (longest first)
            if ch.is_ascii_punctuation() && ch != '_' && ch != '"' && ch != '\'' {
                let remaining: String = chars.clone().take(4).collect();
                let mut matched = false;
                for len in (1..=remaining.len().min(4)).rev() {
                    let candidate = &remaining[..len];
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

            // Identifiers/keywords
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

            // String literals - skip
            if ch == '"' || ch == '\'' {
                chars.next();
                let quote = ch;
                while let Some(&c) = chars.peek() {
                    chars.next();
                    if c == '\\' {
                        chars.next(); // skip escaped char
                    } else if c == quote {
                        break;
                    }
                }
                total_operands += 1; // count the string literal as one operand
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

/// Build aggregated Halstead metrics from source files.
pub(crate) fn build_halstead_report(
    root: &Path,
    files: &[PathBuf],
    export: &ExportData,
    limits: &AnalysisLimits,
) -> Result<HalsteadMetrics> {
    let mut row_map: BTreeMap<String, &FileRow> = BTreeMap::new();
    for row in export.rows.iter().filter(|r| r.kind == FileKind::Parent) {
        row_map.insert(normalize_path(&row.path, root), row);
    }

    let mut all_operators: BTreeMap<String, usize> = BTreeMap::new();
    let mut all_operands: BTreeSet<String> = BTreeSet::new();
    let mut total_ops = 0usize;
    let mut total_opds = 0usize;
    let mut total_bytes = 0u64;
    let max_total = limits.max_bytes;
    let per_file_limit = limits.max_file_bytes.unwrap_or(DEFAULT_MAX_FILE_BYTES) as usize;

    for rel in files {
        if max_total.is_some_and(|limit| total_bytes >= limit) {
            break;
        }
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        let row = match row_map.get(&rel_str) {
            Some(r) => *r,
            None => continue,
        };
        if !is_halstead_lang(&row.lang) {
            continue;
        }

        let path = root.join(rel);
        let bytes = match tokmd_content::read_head(&path, per_file_limit) {
            Ok(b) => b,
            Err(_) => continue,
        };
        total_bytes += bytes.len() as u64;

        if !tokmd_content::is_text_like(&bytes) {
            continue;
        }

        let text = String::from_utf8_lossy(&bytes);
        let lang_lower = row.lang.to_lowercase();
        let counts = tokenize_for_halstead(&text, &lang_lower);

        for (op, count) in counts.operators {
            *all_operators.entry(op).or_insert(0) += count;
        }
        all_operands.extend(counts.operands);
        total_ops += counts.total_operators;
        total_opds += counts.total_operands;
    }

    let n1 = all_operators.len(); // distinct operators
    let n2 = all_operands.len(); // distinct operands
    let vocabulary = n1 + n2;
    let length = total_ops + total_opds;

    let volume = if vocabulary > 0 {
        length as f64 * (vocabulary as f64).log2()
    } else {
        0.0
    };

    let difficulty = if n2 > 0 {
        (n1 as f64 / 2.0) * (total_opds as f64 / n2 as f64)
    } else {
        0.0
    };

    let effort = difficulty * volume;
    let time_seconds = effort / 18.0;
    let estimated_bugs = volume / 3000.0;

    Ok(HalsteadMetrics {
        distinct_operators: n1,
        distinct_operands: n2,
        total_operators: total_ops,
        total_operands: total_opds,
        vocabulary,
        length,
        volume: round_f64(volume, 2),
        difficulty: round_f64(difficulty, 2),
        effort: round_f64(effort, 2),
        time_seconds: round_f64(time_seconds, 2),
        estimated_bugs: round_f64(estimated_bugs, 4),
    })
}

fn round_f64(val: f64, decimals: u32) -> f64 {
    let factor = 10f64.powi(decimals as i32);
    (val * factor).round() / factor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_rust() {
        let code = r#"
fn add(a: i32, b: i32) -> i32 {
    if a > b {
        a + b
    } else {
        a - b
    }
}
"#;
        let counts = tokenize_for_halstead(code, "rust");
        assert!(counts.total_operators > 0);
        assert!(counts.total_operands > 0);
        assert!(!counts.operators.is_empty());
        assert!(!counts.operands.is_empty());
    }

    #[test]
    fn test_tokenize_python() {
        let code = r#"
def add(a, b):
    if a > b:
        return a + b
    else:
        return a - b
"#;
        let counts = tokenize_for_halstead(code, "python");
        assert!(counts.total_operators > 0);
        assert!(counts.total_operands > 0);
    }

    #[test]
    fn test_halstead_computation() {
        // Known values: 2 distinct operators, 3 distinct operands
        // n1=2, n2=3, N1=4, N2=6
        // vocabulary = 5, length = 10
        // volume = 10 * log2(5) ≈ 23.22
        // difficulty = (2/2) * (6/3) = 2.0
        // effort = 2.0 * 23.22 ≈ 46.44
        let n1 = 2;
        let n2 = 3;
        let total_ops = 4;
        let total_opds = 6;
        let vocabulary = n1 + n2;
        let length = total_ops + total_opds;
        let volume = length as f64 * (vocabulary as f64).log2();
        let difficulty = (n1 as f64 / 2.0) * (total_opds as f64 / n2 as f64);
        let effort = difficulty * volume;

        assert!((volume - 23.22).abs() < 0.1);
        assert!((difficulty - 2.0).abs() < 0.01);
        assert!((effort - 46.44).abs() < 0.2);
    }

    #[test]
    fn test_empty_input() {
        let counts = tokenize_for_halstead("", "rust");
        assert_eq!(counts.total_operators, 0);
        assert_eq!(counts.total_operands, 0);
    }
}
