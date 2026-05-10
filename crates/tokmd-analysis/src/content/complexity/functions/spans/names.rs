//! Function name extraction helpers.

/// Extract function name from the line where function starts.
pub(in crate::content::complexity) fn extract_function_name(
    lines: &[&str],
    start_line: usize,
    lang: &str,
) -> String {
    let line = lines.get(start_line).unwrap_or(&"");

    match lang {
        "rust" | "rs" => {
            if let Some(pos) = line.find("fn ") {
                let after_fn = &line[pos + 3..];
                return extract_identifier(after_fn);
            }
        }
        "python" | "py" => {
            if let Some(pos) = line.find("def ") {
                let after_def = &line[pos + 4..];
                return extract_identifier(after_def);
            }
        }
        "javascript" | "js" | "typescript" | "ts" | "jsx" | "tsx" => {
            if let Some(pos) = line.find("function ") {
                let after_func = &line[pos + 9..];
                return extract_identifier(after_func);
            }
            if let Some(pos) = line.find("const ") {
                let after_const = &line[pos + 6..];
                return extract_identifier(after_const);
            }
            if let Some(pos) = line.find("let ") {
                let after_let = &line[pos + 4..];
                return extract_identifier(after_let);
            }
            let trimmed = line.trim();
            if let Some(paren_pos) = trimmed.find('(') {
                let before_paren = &trimmed[..paren_pos];
                let words: Vec<&str> = before_paren.split_whitespace().collect();
                if let Some(last) = words.last() {
                    return (*last).to_string();
                }
            }
        }
        "go" => {
            if let Some(pos) = line.find("func ") {
                let after_func = &line[pos + 5..];
                return extract_identifier(after_func);
            }
        }
        _ => {}
    }

    "unknown".to_string()
}

/// Extract identifier from start of string.
fn extract_identifier(s: &str) -> String {
    let mut name = String::new();
    let mut started = false;

    for ch in s.chars() {
        if !started {
            if ch.is_alphabetic() || ch == '_' {
                started = true;
                name.push(ch);
            }
        } else if ch.is_alphanumeric() || ch == '_' {
            name.push(ch);
        } else {
            break;
        }
    }

    if name.is_empty() {
        "unknown".to_string()
    } else {
        name
    }
}
