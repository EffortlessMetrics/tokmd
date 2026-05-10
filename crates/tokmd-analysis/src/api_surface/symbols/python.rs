use super::{Symbol, has_doc_comment};

pub(super) fn extract_symbols(lines: &[&str]) -> Vec<Symbol> {
    let mut symbols = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Only consider top-level items (no leading whitespace)
        if line.starts_with(' ') || line.starts_with('\t') {
            continue;
        }
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }

        let is_symbol = trimmed.starts_with("def ")
            || trimmed.starts_with("async def ")
            || trimmed.starts_with("class ");

        if is_symbol {
            let name = extract_name(trimmed);
            let is_public = !name.starts_with('_');
            let documented = has_docstring(lines, i);
            symbols.push(Symbol {
                is_public,
                is_documented: documented || has_doc_comment(lines, i),
            });
        }
    }

    symbols
}

fn extract_name(trimmed: &str) -> String {
    let rest = if let Some(r) = trimmed.strip_prefix("async def ") {
        r
    } else if let Some(r) = trimmed.strip_prefix("def ") {
        r
    } else if let Some(r) = trimmed.strip_prefix("class ") {
        r
    } else {
        return String::new();
    };

    rest.chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect()
}

/// Check if the line after the def/class has a docstring.
fn has_docstring(lines: &[&str], idx: usize) -> bool {
    // Look for a docstring in the lines following the definition
    for line in lines.iter().take((idx + 3).min(lines.len())).skip(idx + 1) {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        return t.starts_with("\"\"\"") || t.starts_with("'''") || t.starts_with("r\"\"\"");
    }
    false
}
