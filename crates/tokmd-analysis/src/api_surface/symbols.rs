//! Lightweight multi-language symbol scanning for API surface reports.
//!
//! This module owns heuristic source scanning only. The parent module owns
//! receipt aggregation and stable report construction.

#[cfg(test)]
mod tests;

/// Languages supported for API surface analysis.
pub(super) fn is_api_surface_lang(lang: &str) -> bool {
    matches!(
        lang.to_lowercase().as_str(),
        "rust" | "javascript" | "typescript" | "python" | "go" | "java"
    )
}

/// Represents a single discovered symbol.
#[derive(Debug)]
pub(super) struct Symbol {
    pub(super) is_public: bool,
    pub(super) is_documented: bool,
}

/// Scan a file for public/internal symbols and documentation.
pub(super) fn extract_symbols(lang: &str, text: &str) -> Vec<Symbol> {
    let lines: Vec<&str> = text.lines().collect();
    match lang.to_lowercase().as_str() {
        "rust" => extract_rust_symbols(&lines),
        "javascript" | "typescript" => extract_js_ts_symbols(&lines),
        "python" => extract_python_symbols(&lines),
        "go" => extract_go_symbols(&lines),
        "java" => extract_java_symbols(&lines),
        _ => Vec::new(),
    }
}

/// Check whether the line preceding a symbol looks like a doc comment.
pub(super) fn has_doc_comment(lines: &[&str], idx: usize) -> bool {
    if idx == 0 {
        return false;
    }
    let prev = lines[idx - 1].trim();
    // Rust: /// or //! or #[doc
    // JS/TS/Java: /** or //
    // Python: """ or ''' (handled separately)
    // Go: // directly before declaration
    prev.starts_with("///")
        || prev.starts_with("//!")
        || prev.starts_with("/**")
        || prev.starts_with("#[doc")
        || prev.starts_with("/// ")
        || prev.starts_with("// ")
        || prev.starts_with("\"\"\"")
        || prev.starts_with("'''")
}

// -------
// Rust
// -------

fn extract_rust_symbols(lines: &[&str]) -> Vec<Symbol> {
    let mut symbols = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        // Skip lines inside string literals or comments (simple heuristic)
        if trimmed.starts_with("//") || trimmed.starts_with('*') || trimmed.starts_with("/*") {
            continue;
        }

        let is_public = is_rust_pub_item(trimmed);
        let is_internal = is_rust_internal_item(trimmed);

        if is_public || is_internal {
            symbols.push(Symbol {
                is_public,
                is_documented: has_doc_comment(lines, i),
            });
        }
    }

    symbols
}

fn is_rust_pub_item(trimmed: &str) -> bool {
    // Match pub items, including pub(crate), pub(super), pub(in ...)
    if !trimmed.starts_with("pub ") && !trimmed.starts_with("pub(") {
        return false;
    }

    // Find the part after the pub qualifier
    let after_pub = if trimmed.starts_with("pub(") {
        // Find matching close paren
        if let Some(close) = trimmed.find(')') {
            trimmed[close + 1..].trim_start()
        } else {
            return false;
        }
    } else {
        // "pub " prefix
        &trimmed[4..]
    };

    // Now check for item keywords
    after_pub.starts_with("fn ")
        || after_pub.starts_with("struct ")
        || after_pub.starts_with("enum ")
        || after_pub.starts_with("trait ")
        || after_pub.starts_with("type ")
        || after_pub.starts_with("const ")
        || after_pub.starts_with("static ")
        || after_pub.starts_with("mod ")
        || after_pub.starts_with("async fn ")
        || after_pub.starts_with("unsafe fn ")
        || after_pub.starts_with("unsafe trait ")
}

fn is_rust_internal_item(trimmed: &str) -> bool {
    // Non-pub items at start of line (no leading whitespace for top-level heuristic
    // but we keep it simple: any fn/struct/etc. without pub)
    if trimmed.starts_with("pub ") || trimmed.starts_with("pub(") {
        return false;
    }

    trimmed.starts_with("fn ")
        || trimmed.starts_with("struct ")
        || trimmed.starts_with("enum ")
        || trimmed.starts_with("trait ")
        || trimmed.starts_with("type ")
        || trimmed.starts_with("const ")
        || trimmed.starts_with("static ")
        || trimmed.starts_with("mod ")
        || trimmed.starts_with("async fn ")
        || trimmed.starts_with("unsafe fn ")
        || trimmed.starts_with("unsafe trait ")
}

// -------
// JS/TS
// -------

fn extract_js_ts_symbols(lines: &[&str]) -> Vec<Symbol> {
    let mut symbols = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with("//") || trimmed.starts_with('*') || trimmed.starts_with("/*") {
            continue;
        }

        let is_public = is_js_export(trimmed);
        let is_internal = !is_public && is_js_internal(trimmed);

        if is_public || is_internal {
            symbols.push(Symbol {
                is_public,
                is_documented: has_doc_comment(lines, i),
            });
        }
    }

    symbols
}

fn is_js_export(trimmed: &str) -> bool {
    trimmed.starts_with("export function ")
        || trimmed.starts_with("export async function ")
        || trimmed.starts_with("export class ")
        || trimmed.starts_with("export const ")
        || trimmed.starts_with("export let ")
        || trimmed.starts_with("export default ")
        || trimmed.starts_with("export interface ")
        || trimmed.starts_with("export type ")
        || trimmed.starts_with("export enum ")
        || trimmed.starts_with("export abstract class ")
}

fn is_js_internal(trimmed: &str) -> bool {
    trimmed.starts_with("function ")
        || trimmed.starts_with("async function ")
        || trimmed.starts_with("class ")
        || trimmed.starts_with("const ")
        || trimmed.starts_with("let ")
        || trimmed.starts_with("interface ")
        || trimmed.starts_with("type ")
        || trimmed.starts_with("enum ")
}

// -------
// Python
// -------

fn extract_python_symbols(lines: &[&str]) -> Vec<Symbol> {
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
            let name = extract_python_name(trimmed);
            let is_public = !name.starts_with('_');
            let documented = has_python_docstring(lines, i);
            symbols.push(Symbol {
                is_public,
                is_documented: documented || has_doc_comment(lines, i),
            });
        }
    }

    symbols
}

fn extract_python_name(trimmed: &str) -> String {
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
fn has_python_docstring(lines: &[&str], idx: usize) -> bool {
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

// -------
// Go
// -------

fn extract_go_symbols(lines: &[&str]) -> Vec<Symbol> {
    let mut symbols = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with("//") || trimmed.starts_with("/*") {
            continue;
        }

        if let Some(name) = extract_go_item_name(trimmed) {
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

fn extract_go_item_name(trimmed: &str) -> Option<String> {
    // func Name or func (receiver) Name
    if let Some(rest) = trimmed.strip_prefix("func ") {
        let rest = if rest.starts_with('(') {
            // Method receiver: skip to closing paren
            if let Some(close) = rest.find(')') {
                rest[close + 1..].trim_start()
            } else {
                return None;
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

// -------
// Java
// -------

fn extract_java_symbols(lines: &[&str]) -> Vec<Symbol> {
    let mut symbols = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with("//") || trimmed.starts_with('*') || trimmed.starts_with("/*") {
            continue;
        }

        let is_public = is_java_public(trimmed);
        let is_internal = !is_public && is_java_internal(trimmed);

        if is_public || is_internal {
            symbols.push(Symbol {
                is_public,
                is_documented: has_doc_comment(lines, i),
            });
        }
    }

    symbols
}

fn is_java_public(trimmed: &str) -> bool {
    trimmed.starts_with("public class ")
        || trimmed.starts_with("public interface ")
        || trimmed.starts_with("public enum ")
        || trimmed.starts_with("public static ")
        || trimmed.starts_with("public abstract class ")
        || trimmed.starts_with("public final class ")
        || trimmed.starts_with("public record ")
        || trimmed.starts_with("public sealed ")
        // public return-type method(
        || (trimmed.starts_with("public ")
            && (trimmed.contains('(') || trimmed.contains(" class ") || trimmed.contains(" interface ")))
}

fn is_java_internal(trimmed: &str) -> bool {
    // private/protected/package-private items
    trimmed.starts_with("private ")
        || trimmed.starts_with("protected ")
        || trimmed.starts_with("class ")
        || trimmed.starts_with("interface ")
        || trimmed.starts_with("enum ")
        || trimmed.starts_with("abstract class ")
        || trimmed.starts_with("final class ")
        || trimmed.starts_with("static ")
        || trimmed.starts_with("record ")
}
