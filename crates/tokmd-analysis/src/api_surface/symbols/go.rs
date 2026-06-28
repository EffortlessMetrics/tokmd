use super::{Symbol, has_doc_comment};

pub(super) fn extract_symbols(lines: &[&str]) -> Vec<Symbol> {
    let mut symbols = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with("//") || trimmed.starts_with("/*") {
            continue;
        }

        if let Some(name) = extract_item_name(trimmed) {
            // In Go, items starting with uppercase are public
            let first_char = name.chars().next().unwrap_or('_');
            let is_public = first_char.is_uppercase();
            symbols.push(Symbol {
                is_public,
                is_documented: has_doc_comment(lines, i),
            });
        }
    }

    symbols
}

fn extract_item_name(trimmed: &str) -> Option<String> {
    // func Name or func (receiver) Name
    if let Some(rest) = trimmed.strip_prefix("func ") {
        let rest = if rest.starts_with('(') {
            // Method receiver: skip to closing paren
            {
                let close = rest.find(')')?;
                rest[close + 1..].trim_start()
            }
        } else {
            rest
        };
        let name: String = rest
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if !name.is_empty() {
            return Some(name);
        }
    }

    // type Name struct/interface
    if let Some(rest) = trimmed.strip_prefix("type ") {
        let name: String = rest
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if !name.is_empty() {
            return Some(name);
        }
    }

    // var Name or const Name (top-level)
    for prefix in &["var ", "const "] {
        if let Some(rest) = trimmed.strip_prefix(prefix) {
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                return Some(name);
            }
        }
    }

    None
}
