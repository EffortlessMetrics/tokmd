//! Language-specific lightweight complexity heuristics for handoff intelligence.

/// Check if a language is analyzable for complexity.
pub(super) fn is_analyzable_lang(lang: &str) -> bool {
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

/// Count functions and estimate max function length in lines.
/// Simplified inline version that avoids heavy dependencies.
pub(super) fn count_functions_simple(lang: &str, text: &str) -> (usize, usize) {
    let lines: Vec<&str> = text.lines().collect();
    match lang.to_lowercase().as_str() {
        "rust" => count_brace_functions(&lines, is_rust_fn_start_simple),
        "go" => count_brace_functions(&lines, |t| t.starts_with("func ")),
        "javascript" | "typescript" => count_brace_functions(&lines, |t| {
            t.starts_with("function ")
                || t.starts_with("async function ")
                || t.starts_with("export function ")
                || t.starts_with("export async function ")
                || (t.contains("=> {") && !t.starts_with("//"))
        }),
        "c" | "c++" | "java" | "c#" | "php" => count_brace_functions(&lines, |t| {
            (t.ends_with(") {") || t.ends_with("){"))
                && !t.starts_with("if ")
                && !t.starts_with("if(")
                && !t.starts_with("while ")
                && !t.starts_with("while(")
                && !t.starts_with("for ")
                && !t.starts_with("for(")
                && !t.starts_with("switch ")
                && !t.starts_with("//")
        }),
        "python" => count_python_functions_simple(&lines),
        "ruby" => count_ruby_functions_simple(&lines),
        _ => (0, 0),
    }
}

/// Estimate file-level cyclomatic complexity by counting branching keywords.
pub(super) fn estimate_cyclomatic_simple(lang: &str, text: &str) -> usize {
    let mut complexity: usize = 1; // base

    let keywords: &[&str] = match lang.to_lowercase().as_str() {
        "rust" => &[
            "if ", "else if ", "match ", "while ", "for ", "loop ", "&&", "||",
        ],
        "javascript" | "typescript" => &[
            "if ", "else if ", "switch ", "case ", "while ", "for ", "&&", "||", "catch ",
        ],
        "python" => &["if ", "elif ", "while ", "for ", "except ", " and ", " or "],
        "go" => &[
            "if ", "else if ", "switch ", "case ", "for ", "select ", "&&", "||",
        ],
        "c" | "c++" | "java" | "c#" | "php" => &[
            "if ", "else if ", "switch ", "case ", "while ", "for ", "&&", "||", "catch ",
        ],
        "ruby" => &[
            "if ", "elsif ", "unless ", "while ", "until ", "for ", "case ", "when ", "rescue ",
        ],
        _ => return 1,
    };

    for line in text.lines() {
        let trimmed = line.trim();
        // Skip comments.
        if trimmed.starts_with("//") || trimmed.starts_with('#') || trimmed.starts_with("/*") {
            continue;
        }
        for keyword in keywords {
            complexity += trimmed.matches(keyword).count();
        }
    }

    complexity
}

/// Check if a trimmed line starts a Rust function definition.
/// Handles all visibility qualifiers including `pub(in path)`, extern "ABI", etc.
fn is_rust_fn_start_simple(trimmed: &str) -> bool {
    let Some(fn_pos) = trimmed.find("fn ") else {
        return false;
    };
    let prefix = trimmed[..fn_pos].trim();
    if prefix.is_empty() {
        return true;
    }
    let mut rest = prefix;
    while !rest.is_empty() {
        rest = rest.trim_start();
        if rest.is_empty() {
            break;
        }
        if rest.starts_with("pub(") {
            if let Some(close) = rest.find(')') {
                rest = &rest[close + 1..];
            } else {
                return false;
            }
        } else if let Some(r) = rest.strip_prefix("pub") {
            rest = r;
        } else if let Some(r) = rest.strip_prefix("async") {
            rest = r;
        } else if let Some(r) = rest.strip_prefix("unsafe") {
            rest = r;
        } else if let Some(r) = rest.strip_prefix("const") {
            rest = r;
        } else if rest.starts_with("extern") {
            rest = rest["extern".len()..].trim_start();
            if rest.starts_with('"') {
                if let Some(close) = rest[1..].find('"') {
                    rest = &rest[close + 2..];
                } else {
                    return false;
                }
            }
        } else {
            return false;
        }
    }
    true
}

/// Count functions in brace-delimited languages.
fn count_brace_functions(lines: &[&str], is_fn_start: impl Fn(&str) -> bool) -> (usize, usize) {
    let mut count = 0;
    let mut max_len = 0;
    let mut in_fn = false;
    let mut fn_start = 0;
    let mut brace_depth: usize = 0;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if !in_fn && is_fn_start(trimmed) {
            count += 1;
            in_fn = true;
            fn_start = i;
            brace_depth = 0;
        }
        if in_fn {
            brace_depth += line.chars().filter(|&c| c == '{').count();
            brace_depth = brace_depth.saturating_sub(line.chars().filter(|&c| c == '}').count());
            if brace_depth == 0 && line.contains('}') {
                let fn_len = i - fn_start + 1;
                max_len = max_len.max(fn_len);
                in_fn = false;
            }
        }
    }

    (count, max_len)
}

/// Count functions in Python (indentation-based).
fn count_python_functions_simple(lines: &[&str]) -> (usize, usize) {
    let mut count = 0;
    let mut max_len = 0;
    let mut fn_start = 0;
    let mut fn_indent = 0;
    let mut in_fn = false;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("def ") || trimmed.starts_with("async def ") {
            if in_fn {
                max_len = max_len.max(i - fn_start);
            }
            count += 1;
            in_fn = true;
            fn_start = i;
            fn_indent = line.len() - line.trim_start().len();
        } else if in_fn && !trimmed.is_empty() && !trimmed.starts_with('#') {
            let indent = line.len() - line.trim_start().len();
            if indent <= fn_indent
                && !trimmed.starts_with("def ")
                && !trimmed.starts_with("async def ")
            {
                max_len = max_len.max(i - fn_start);
                in_fn = false;
            }
        }
    }
    if in_fn {
        max_len = max_len.max(lines.len() - fn_start);
    }

    (count, max_len)
}

/// Count functions in Ruby (end-delimited).
fn count_ruby_functions_simple(lines: &[&str]) -> (usize, usize) {
    let mut count = 0;
    let mut max_len = 0;
    let mut fn_start = 0;
    let mut in_fn = false;
    let mut depth = 0;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("def ") {
            if !in_fn {
                count += 1;
                in_fn = true;
                fn_start = i;
                depth = 1;
            } else {
                depth += 1;
            }
        } else if in_fn {
            if trimmed.starts_with("do")
                || trimmed.starts_with("class ")
                || trimmed.starts_with("module ")
                || trimmed.starts_with("begin")
                || trimmed.starts_with("if ")
                || trimmed.starts_with("unless ")
                || trimmed.starts_with("case ")
            {
                depth += 1;
            }
            if trimmed == "end" || trimmed.starts_with("end ") {
                depth -= 1;
                if depth == 0 {
                    max_len = max_len.max(i - fn_start + 1);
                    in_fn = false;
                }
            }
        }
    }

    (count, max_len)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_functions_simple_rust() {
        let code = r#"
fn simple() {
    println!("hello");
}

pub fn public_fn() {
    let x = 1;
    let y = 2;
}

pub async fn async_fn() {
    todo!()
}
"#;
        let (count, max_len) = count_functions_simple("Rust", code);
        assert_eq!(count, 3);
        assert!(max_len >= 3);
    }

    #[test]
    fn test_count_functions_simple_python() {
        let code = r#"
def foo():
    pass

async def bar():
    await something()

def baz():
    x = 1
    y = 2
    return x + y
"#;
        let (count, _max_len) = count_functions_simple("Python", code);
        assert_eq!(count, 3);
    }

    #[test]
    fn test_estimate_cyclomatic_simple_rust() {
        let code = r#"
fn complex(x: i32) -> i32 {
    if x > 0 {
        if x > 10 {
            x * 2
        } else {
            x + 1
        }
    } else {
        match x {
            -1 => 0,
            _ => x.abs(),
        }
    }
}
"#;
        let cyclo = estimate_cyclomatic_simple("Rust", code);
        // Base 1 + 2 ifs + 1 else if (none here) + 1 match = 4+.
        assert!(cyclo >= 4, "Expected cyclomatic >= 4, got {}", cyclo);
    }

    #[test]
    fn test_is_analyzable_lang() {
        assert!(is_analyzable_lang("Rust"));
        assert!(is_analyzable_lang("javascript"));
        assert!(is_analyzable_lang("Python"));
        assert!(!is_analyzable_lang("Markdown"));
        assert!(!is_analyzable_lang("JSON"));
        assert!(!is_analyzable_lang("TOML"));
    }
}
