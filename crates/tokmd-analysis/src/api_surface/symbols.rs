//! Lightweight multi-language symbol scanning for API surface reports.
//!
//! This module owns heuristic source scanning only. The parent module owns
//! receipt aggregation and stable report construction.

#[cfg(test)]
mod tests;

mod go;
mod java;
mod js_ts;
mod python;
mod rust;

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
        "rust" => rust::extract_symbols(&lines),
        "javascript" | "typescript" => js_ts::extract_symbols(&lines),
        "python" => python::extract_symbols(&lines),
        "go" => go::extract_symbols(&lines),
        "java" => java::extract_symbols(&lines),
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
