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

#[cfg(test)]
mod tests {
    use super::*;

    // ---- supports_language ----

    #[test]
    fn test_supports_known_languages() {
        assert!(supports_language("Rust"));
        assert!(supports_language("rust"));
        assert!(supports_language("JavaScript"));
        assert!(supports_language("TypeScript"));
        assert!(supports_language("Python"));
        assert!(supports_language("Go"));
    }

    #[test]
    fn test_unsupported_languages() {
        assert!(!supports_language("Java"));
        assert!(!supports_language("C"));
        assert!(!supports_language("C++"));
        assert!(!supports_language("Ruby"));
        assert!(!supports_language(""));
    }

    // ---- normalize_import_target ----

    #[test]
    fn test_normalize_relative_to_local() {
        assert_eq!(normalize_import_target("./utils"), "local");
        assert_eq!(normalize_import_target("../lib"), "local");
        assert_eq!(normalize_import_target("."), "local");
    }

    #[test]
    fn test_normalize_npm_scoped_package() {
        assert_eq!(normalize_import_target("react/dom"), "react");
        assert_eq!(normalize_import_target("lodash/fp"), "lodash");
    }

    #[test]
    fn test_normalize_rust_crate() {
        assert_eq!(normalize_import_target("std::collections"), "std");
        assert_eq!(normalize_import_target("serde::Deserialize"), "serde");
    }

    #[test]
    fn test_normalize_python_dotted() {
        assert_eq!(normalize_import_target("os.path"), "os");
        assert_eq!(normalize_import_target("collections.abc"), "collections");
    }

    #[test]
    fn test_normalize_strips_quotes() {
        assert_eq!(normalize_import_target("\"react\""), "react");
        assert_eq!(normalize_import_target("'lodash'"), "lodash");
    }

    #[test]
    fn test_normalize_trims_whitespace() {
        assert_eq!(normalize_import_target("  react  "), "react");
    }

    // ---- parse_imports: Rust ----

    #[test]
    fn test_parse_rust_use_statement() {
        let lines = ["use std::collections::HashMap;", "use serde::Deserialize;"];
        let imports = parse_imports("Rust", &lines);
        assert_eq!(imports, vec!["std", "serde"]);
    }

    #[test]
    fn test_parse_rust_mod_statement() {
        let lines = ["mod utils;", "mod tests;"];
        let imports = parse_imports("Rust", &lines);
        assert_eq!(imports, vec!["utils", "tests"]);
    }

    #[test]
    fn test_parse_rust_mixed() {
        let lines = [
            "use anyhow::Result;",
            "mod config;",
            "fn main() {}",
            "use tokei::Languages;",
        ];
        let imports = parse_imports("rust", &lines);
        assert_eq!(imports, vec!["anyhow", "config", "tokei"]);
    }

    #[test]
    fn test_parse_rust_ignores_non_import_lines() {
        let lines = ["fn main() {}", "let x = 42;", "// use fake;"];
        let imports = parse_imports("Rust", &lines);
        assert!(imports.is_empty());
    }

    // ---- parse_imports: JavaScript/TypeScript ----

    #[test]
    fn test_parse_js_import_from() {
        let lines = [
            "import React from 'react';",
            "import { useState } from \"react\";",
        ];
        let imports = parse_imports("JavaScript", &lines);
        assert_eq!(imports, vec!["react", "react"]);
    }

    #[test]
    fn test_parse_js_require() {
        let lines = [
            "const fs = require('fs');",
            "const path = require(\"path\");",
        ];
        let imports = parse_imports("JavaScript", &lines);
        assert_eq!(imports, vec!["fs", "path"]);
    }

    #[test]
    fn test_parse_ts_imports() {
        let lines = ["import type { Foo } from 'bar';"];
        let imports = parse_imports("TypeScript", &lines);
        assert_eq!(imports, vec!["bar"]);
    }

    // ---- parse_imports: Python ----

    #[test]
    fn test_parse_python_import() {
        let lines = ["import os", "import sys"];
        let imports = parse_imports("Python", &lines);
        assert_eq!(imports, vec!["os", "sys"]);
    }

    #[test]
    fn test_parse_python_from_import() {
        let lines = [
            "from pathlib import Path",
            "from collections import defaultdict",
        ];
        let imports = parse_imports("Python", &lines);
        assert_eq!(imports, vec!["pathlib", "collections"]);
    }

    #[test]
    fn test_parse_python_ignores_comments() {
        let lines = ["# import fake", "import os"];
        let imports = parse_imports("Python", &lines);
        assert_eq!(imports, vec!["os"]);
    }

    // ---- parse_imports: Go ----

    #[test]
    fn test_parse_go_single_import() {
        let lines = ["import \"fmt\""];
        let imports = parse_imports("Go", &lines);
        assert_eq!(imports, vec!["fmt"]);
    }

    #[test]
    fn test_parse_go_block_import() {
        let lines = ["import (", "\t\"fmt\"", "\t\"os\"", ")"];
        let imports = parse_imports("Go", &lines);
        assert_eq!(imports, vec!["fmt", "os"]);
    }

    #[test]
    fn test_parse_go_std_and_external() {
        let lines = ["import (", "\t\"fmt\"", "\t\"github.com/pkg/errors\"", ")"];
        let imports = parse_imports("Go", &lines);
        assert_eq!(imports, vec!["fmt", "github.com/pkg/errors"]);
    }

    // ---- parse_imports: unsupported language ----

    #[test]
    fn test_parse_unsupported_language_returns_empty() {
        let lines = ["#include <stdio.h>"];
        let imports = parse_imports("C", &lines);
        assert!(imports.is_empty());
    }

    // ---- parse_imports: empty input ----

    #[test]
    fn test_parse_empty_input() {
        let lines: Vec<&str> = vec![];
        let imports = parse_imports("Rust", &lines);
        assert!(imports.is_empty());
    }

    // ---- extract_quoted ----

    #[test]
    fn test_extract_quoted_double() {
        assert_eq!(extract_quoted("from \"hello\""), Some("hello".to_string()));
    }

    #[test]
    fn test_extract_quoted_single() {
        assert_eq!(extract_quoted("from 'world'"), Some("world".to_string()));
    }

    #[test]
    fn test_extract_quoted_empty_string() {
        assert_eq!(extract_quoted("\"\""), None);
    }

    #[test]
    fn test_extract_quoted_no_quotes() {
        assert_eq!(extract_quoted("no quotes here"), None);
    }
}
