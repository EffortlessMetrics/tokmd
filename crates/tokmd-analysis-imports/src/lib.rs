//! Language-aware import extraction and deterministic target normalization.
//!
//! This crate intentionally keeps only parsing and normalization logic for
//! import-like statements so higher-tier crates can compose it without
//! filesystem or receipt dependencies.

#![forbid(unsafe_code)]

/// Returns true when `lang` supports import extraction.
pub fn supports_language(lang: &str) -> bool {
    matches!(
        lang.to_ascii_lowercase().as_str(),
        "rust" | "javascript" | "typescript" | "python" | "go"
    )
}

/// Extract import-like targets from language-specific source lines.
pub fn parse_imports<S: AsRef<str>>(lang: &str, lines: &[S]) -> Vec<String> {
    match lang.to_ascii_lowercase().as_str() {
        "rust" => parse_rust_imports(lines),
        "javascript" | "typescript" => parse_js_imports(lines),
        "python" => parse_py_imports(lines),
        "go" => parse_go_imports(lines),
        _ => Vec::new(),
    }
}

/// Normalize an import target into a stable dependency root.
///
/// Relative imports are collapsed to `local`.
pub fn normalize_import_target(target: &str) -> String {
    let trimmed = target.trim();
    if trimmed.starts_with('.') {
        return "local".to_string();
    }
    let trimmed = trimmed.trim_matches('"').trim_matches('\'');
    trimmed
        .split(['/', ':', '.'])
        .next()
        .unwrap_or(trimmed)
        .to_string()
}

fn parse_rust_imports<S: AsRef<str>>(lines: &[S]) -> Vec<String> {
    let mut imports = Vec::new();
    for line in lines {
        let trimmed = line.as_ref().trim();
        if trimmed.starts_with("use ")
            && let Some(rest) = trimmed.strip_prefix("use ")
        {
            let rest = rest.trim_end_matches(';').trim();
            let target = rest.split("::").next().unwrap_or(rest).to_string();
            imports.push(target);
        } else if trimmed.starts_with("mod ")
            && let Some(rest) = trimmed.strip_prefix("mod ")
        {
            let target = rest.trim_end_matches(';').trim().to_string();
            imports.push(target);
        }
    }
    imports
}

fn parse_js_imports<S: AsRef<str>>(lines: &[S]) -> Vec<String> {
    let mut imports = Vec::new();
    for line in lines {
        let trimmed = line.as_ref().trim();
        if trimmed.starts_with("import ")
            && let Some(target) = extract_quoted(trimmed)
        {
            imports.push(target);
        }
        if let Some(idx) = trimmed.find("require(")
            && let Some(target) = extract_quoted(&trimmed[idx..])
        {
            imports.push(target);
        }
    }
    imports
}

fn parse_py_imports<S: AsRef<str>>(lines: &[S]) -> Vec<String> {
    let mut imports = Vec::new();
    for line in lines {
        let trimmed = line.as_ref().trim();
        if trimmed.starts_with("import ")
            && let Some(rest) = trimmed.strip_prefix("import ")
        {
            let target = rest.split_whitespace().next().unwrap_or(rest).to_string();
            imports.push(target);
        } else if trimmed.starts_with("from ")
            && let Some(rest) = trimmed.strip_prefix("from ")
        {
            let target = rest.split_whitespace().next().unwrap_or(rest).to_string();
            imports.push(target);
        }
    }
    imports
}

fn parse_go_imports<S: AsRef<str>>(lines: &[S]) -> Vec<String> {
    let mut imports = Vec::new();
    let mut in_block = false;
    for line in lines {
        let trimmed = line.as_ref().trim();
        if trimmed.starts_with("import (") {
            in_block = true;
            continue;
        }
        if in_block {
            if trimmed.starts_with(')') {
                in_block = false;
                continue;
            }
            if let Some(target) = extract_quoted(trimmed) {
                imports.push(target);
            }
            continue;
        }
        if trimmed.starts_with("import ")
            && let Some(target) = extract_quoted(trimmed)
        {
            imports.push(target);
        }
    }
    imports
}

fn extract_quoted(text: &str) -> Option<String> {
    let mut chars = text.chars();
    let mut quote = None;
    for ch in chars.by_ref() {
        if ch == '"' || ch == '\'' {
            quote = Some(ch);
            break;
        }
    }
    let quote = quote?;
    let mut out = String::new();
    for ch in chars {
        if ch == quote {
            break;
        }
        out.push(ch);
    }
    if out.is_empty() { None } else { Some(out) }
}
