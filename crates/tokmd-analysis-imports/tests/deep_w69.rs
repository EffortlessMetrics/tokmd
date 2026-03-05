//! Deep tests for tokmd-analysis-imports (W69).
//!
//! Covers: import extraction for all languages, normalization,
//! supports_language, deterministic ordering, edge cases.

use tokmd_analysis_imports::{normalize_import_target, parse_imports, supports_language};

// ===================================================================
// 1. supports_language
// ===================================================================

#[test]
fn supports_all_documented_languages() {
    for lang in &["Rust", "JavaScript", "TypeScript", "Python", "Go"] {
        assert!(supports_language(lang), "{lang} should be supported");
    }
}

#[test]
fn supports_language_is_case_insensitive() {
    assert!(supports_language("rust"));
    assert!(supports_language("RUST"));
    assert!(supports_language("Rust"));
}

#[test]
fn unsupported_languages_return_false() {
    for lang in &["Java", "C", "C++", "Ruby", "Haskell", ""] {
        assert!(!supports_language(lang), "{lang} should not be supported");
    }
}

// ===================================================================
// 2. Rust imports
// ===================================================================

#[test]
fn rust_use_extracts_crate_root() {
    let lines = ["use std::collections::HashMap;"];
    let result = parse_imports("Rust", &lines);
    assert_eq!(result, vec!["std"]);
}

#[test]
fn rust_mod_extracts_module_name() {
    let lines = ["mod config;"];
    let result = parse_imports("Rust", &lines);
    assert_eq!(result, vec!["config"]);
}

#[test]
fn rust_mixed_use_and_mod() {
    let lines = [
        "use serde::Deserialize;",
        "mod utils;",
        "use anyhow::Result;",
    ];
    let result = parse_imports("Rust", &lines);
    assert_eq!(result, vec!["serde", "utils", "anyhow"]);
}

#[test]
fn rust_ignores_non_import_lines() {
    let lines = [
        "fn main() {}",
        "// use fake;",
        "let x = 42;",
    ];
    let result = parse_imports("Rust", &lines);
    assert!(result.is_empty());
}

// ===================================================================
// 3. JavaScript/TypeScript imports
// ===================================================================

#[test]
fn js_import_from_single_quotes() {
    let lines = ["import React from 'react';"];
    let result = parse_imports("JavaScript", &lines);
    assert_eq!(result, vec!["react"]);
}

#[test]
fn js_import_from_double_quotes() {
    let lines = ["import { useState } from \"react\";"];
    let result = parse_imports("JavaScript", &lines);
    assert_eq!(result, vec!["react"]);
}

#[test]
fn js_require_extracts_target() {
    let lines = ["const fs = require('fs');"];
    let result = parse_imports("JavaScript", &lines);
    assert_eq!(result, vec!["fs"]);
}

#[test]
fn ts_type_import() {
    let lines = ["import type { Config } from 'config';"];
    let result = parse_imports("TypeScript", &lines);
    assert_eq!(result, vec!["config"]);
}

// ===================================================================
// 4. Python imports
// ===================================================================

#[test]
fn python_import_statement() {
    let lines = ["import os", "import sys"];
    let result = parse_imports("Python", &lines);
    assert_eq!(result, vec!["os", "sys"]);
}

#[test]
fn python_from_import() {
    let lines = ["from pathlib import Path"];
    let result = parse_imports("Python", &lines);
    assert_eq!(result, vec!["pathlib"]);
}

#[test]
fn python_ignores_comments() {
    let lines = ["# import fake", "import os"];
    let result = parse_imports("Python", &lines);
    assert_eq!(result, vec!["os"]);
}

// ===================================================================
// 5. Go imports
// ===================================================================

#[test]
fn go_single_import() {
    let lines = ["import \"fmt\""];
    let result = parse_imports("Go", &lines);
    assert_eq!(result, vec!["fmt"]);
}

#[test]
fn go_block_import() {
    let lines = ["import (", "\t\"fmt\"", "\t\"os\"", ")"];
    let result = parse_imports("Go", &lines);
    assert_eq!(result, vec!["fmt", "os"]);
}

#[test]
fn go_external_import() {
    let lines = ["import (", "\t\"github.com/pkg/errors\"", ")"];
    let result = parse_imports("Go", &lines);
    assert_eq!(result, vec!["github.com/pkg/errors"]);
}

// ===================================================================
// 6. normalize_import_target
// ===================================================================

#[test]
fn normalize_relative_imports_to_local() {
    assert_eq!(normalize_import_target("./utils"), "local");
    assert_eq!(normalize_import_target("../lib"), "local");
    assert_eq!(normalize_import_target("."), "local");
}

#[test]
fn normalize_extracts_package_root() {
    assert_eq!(normalize_import_target("react/dom"), "react");
    assert_eq!(normalize_import_target("std::collections"), "std");
    assert_eq!(normalize_import_target("os.path"), "os");
}

#[test]
fn normalize_strips_quotes() {
    assert_eq!(normalize_import_target("\"react\""), "react");
    assert_eq!(normalize_import_target("'lodash'"), "lodash");
}

#[test]
fn normalize_trims_whitespace() {
    assert_eq!(normalize_import_target("  react  "), "react");
}

// ===================================================================
// 7. Deterministic ordering
// ===================================================================

#[test]
fn parse_preserves_insertion_order() {
    let lines = [
        "use z_crate::Foo;",
        "use a_crate::Bar;",
        "use m_crate::Baz;",
    ];
    let result = parse_imports("Rust", &lines);
    assert_eq!(result, vec!["z_crate", "a_crate", "m_crate"]);
}

#[test]
fn parse_is_deterministic() {
    let lines = [
        "import os",
        "from pathlib import Path",
        "import sys",
    ];
    let a = parse_imports("Python", &lines);
    let b = parse_imports("Python", &lines);
    assert_eq!(a, b, "parse_imports must be deterministic");
}

// ===================================================================
// 8. Edge cases
// ===================================================================

#[test]
fn empty_input_returns_empty() {
    let lines: Vec<&str> = vec![];
    assert!(parse_imports("Rust", &lines).is_empty());
    assert!(parse_imports("Python", &lines).is_empty());
    assert!(parse_imports("JavaScript", &lines).is_empty());
    assert!(parse_imports("Go", &lines).is_empty());
}

#[test]
fn unsupported_language_returns_empty() {
    let lines = ["#include <stdio.h>"];
    assert!(parse_imports("C", &lines).is_empty());
}
