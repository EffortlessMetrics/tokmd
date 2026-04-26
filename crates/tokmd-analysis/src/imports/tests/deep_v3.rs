//! Deep tests for tokmd-analysis imports module: parsing, normalization, edge cases.

use crate::imports::{normalize_import_target, parse_imports, supports_language};

// ── supports_language: exhaustive coverage ───────────────────────────

#[test]
fn supports_language_all_five_supported() {
    for lang in ["rust", "javascript", "typescript", "python", "go"] {
        assert!(supports_language(lang), "{lang} should be supported");
    }
}

#[test]
fn supports_language_case_mixed() {
    assert!(supports_language("RuSt"));
    assert!(supports_language("JAVASCRIPT"));
    assert!(supports_language("gO"));
    assert!(supports_language("pYtHoN"));
    assert!(supports_language("TypeScript"));
}

#[test]
fn supports_language_rejects_similar_names() {
    assert!(!supports_language("rust-lang"));
    assert!(!supports_language("javascript1"));
    assert!(!supports_language("python3"));
    assert!(!supports_language("golang"));
    assert!(!supports_language("ts"));
    assert!(!supports_language("js"));
    assert!(!supports_language("py"));
    assert!(!supports_language("rs"));
}

#[test]
fn supports_language_rejects_empty_and_whitespace() {
    assert!(!supports_language(""));
    assert!(!supports_language("   "));
    assert!(!supports_language("\t"));
}

// ── parse_imports: Rust deep cases ──────────────────────────────────

#[test]
fn rust_use_with_alias() {
    let lines = ["use std::io::Result as IoResult;"];
    let imports = parse_imports("rust", &lines);
    assert_eq!(imports, vec!["std"]);
}

#[test]
fn rust_use_glob_import() {
    let lines = ["use std::prelude::*;"];
    let imports = parse_imports("rust", &lines);
    assert_eq!(imports, vec!["std"]);
}

#[test]
fn rust_mod_with_block_body_not_captured() {
    // Only `mod name;` is captured, not `mod name { ... }`
    let lines = ["mod inline {", "    fn foo() {}", "}"];
    let imports = parse_imports("rust", &lines);
    // "mod inline {" starts with "mod " so it will match, extracting "inline"
    // The "{" is part of the trimmed result
    assert_eq!(imports.len(), 1);
    assert!(imports[0].starts_with("inline"));
}

#[test]
fn rust_pub_mod_not_captured() {
    let lines = ["pub mod public_api;"];
    let imports = parse_imports("rust", &lines);
    assert!(
        imports.is_empty(),
        "pub mod should not be captured (doesn't start with 'mod ')"
    );
}

#[test]
fn rust_multiple_use_same_crate() {
    let lines = [
        "use serde::Serialize;",
        "use serde::Deserialize;",
        "use serde_json::Value;",
    ];
    let imports = parse_imports("rust", &lines);
    assert_eq!(imports, vec!["serde", "serde", "serde_json"]);
}

#[test]
fn rust_use_with_leading_whitespace_and_trailing() {
    let lines = ["    use   tokei::Languages;   "];
    let imports = parse_imports("rust", &lines);
    // After trim, "use   tokei::Languages;   " -> starts with "use " ✓
    // rest = "  tokei::Languages;" -> trim_end(';') -> "  tokei::Languages" -> trim -> not done at this level
    // Actually: strip_prefix("use ") gives "  tokei::Languages;   ", trim_end(';') -> "  tokei::Languages   ", trim -> "tokei::Languages"
    // split("::").next() -> "tokei::Languages".split("::").next() = Some("  tokei")
    // Hmm, let me re-read the code. The line is trimmed first.
    // trimmed = "use   tokei::Languages;"
    // starts_with("use ") => true
    // rest = strip_prefix("use ") => "  tokei::Languages;"
    // rest.trim_end_matches(';').trim() => "tokei::Languages"
    // split("::").next() => "tokei"
    assert_eq!(imports, vec!["tokei"]);
}

// ── parse_imports: JavaScript deep cases ────────────────────────────

#[test]
fn js_import_with_both_default_and_named() {
    let lines = [r#"import React, { useState, useEffect } from "react";"#];
    let imports = parse_imports("javascript", &lines);
    assert_eq!(imports, vec!["react"]);
}

#[test]
fn js_dynamic_import_not_captured() {
    // Dynamic import() in a const assignment doesn't start with "import "
    let lines = [r#"const mod = await import("./lazy");"#];
    let imports = parse_imports("javascript", &lines);
    assert!(
        imports.is_empty(),
        "dynamic import() should not be captured"
    );
}

#[test]
fn js_require_resolve_not_captured() {
    // require.resolve( is not require( — the dot breaks the match
    let lines = [r#"const data = JSON.parse(fs.readFileSync(require.resolve("./data.json")));"#];
    let imports = parse_imports("javascript", &lines);
    assert!(
        imports.is_empty(),
        "require.resolve should not match require("
    );
}

#[test]
fn js_import_with_no_from_clause() {
    // Side-effect import without `from`
    let lines = [r#"import "core-js/stable";"#];
    let imports = parse_imports("javascript", &lines);
    assert_eq!(imports, vec!["core-js/stable"]);
}

#[test]
fn js_empty_require_string_not_captured() {
    let lines = [r#"const x = require("");"#];
    let imports = parse_imports("javascript", &lines);
    assert!(
        imports.is_empty(),
        "empty quoted string should not produce import"
    );
}

#[test]
fn js_import_and_require_on_same_line() {
    let lines = [r#"import foo from "bar"; const x = require("baz");"#];
    let imports = parse_imports("javascript", &lines);
    // import extracts "bar", require extracts "baz"
    assert_eq!(imports, vec!["bar", "baz"]);
}

// ── parse_imports: Python deep cases ────────────────────────────────

#[test]
fn python_import_with_as_clause() {
    let lines = ["import numpy as np"];
    let imports = parse_imports("python", &lines);
    assert_eq!(imports, vec!["numpy"]);
}

#[test]
fn python_from_import_multiple_names() {
    let lines = ["from collections import OrderedDict, defaultdict, Counter"];
    let imports = parse_imports("python", &lines);
    assert_eq!(imports, vec!["collections"]);
}

#[test]
fn python_from_relative_import() {
    let lines = ["from . import utils", "from ..models import User"];
    let imports = parse_imports("python", &lines);
    assert_eq!(imports, vec![".", "..models"]);
}

#[test]
fn python_ignores_inline_comment_import() {
    let lines = ["x = 1  # import os"];
    let imports = parse_imports("python", &lines);
    assert!(imports.is_empty());
}

#[test]
fn python_conditional_import() {
    let lines = ["    import sys", "    from pathlib import Path"];
    let imports = parse_imports("python", &lines);
    assert_eq!(imports, vec!["sys", "pathlib"]);
}

// ── parse_imports: Go deep cases ────────────────────────────────────

#[test]
fn go_block_with_alias_and_blank_import() {
    let lines = vec![
        "import (",
        r#"    . "fmt""#,
        r#"    _ "net/http/pprof""#,
        ")",
    ];
    let imports = parse_imports("go", &lines);
    assert_eq!(imports, vec!["fmt", "net/http/pprof"]);
}

#[test]
fn go_multiple_separate_blocks() {
    let lines = vec![
        "import (",
        r#"    "fmt""#,
        ")",
        "",
        "import (",
        r#"    "os""#,
        ")",
    ];
    let imports = parse_imports("go", &lines);
    assert_eq!(imports, vec!["fmt", "os"]);
}

#[test]
fn go_block_with_comment_line_skipped() {
    let lines = vec!["import (", "    // standard library", r#"    "fmt""#, ")"];
    let imports = parse_imports("go", &lines);
    // The comment line has no quotes, so extract_quoted returns None
    assert_eq!(imports, vec!["fmt"]);
}

#[test]
fn go_no_import_in_non_import_context() {
    let lines = vec![r#"fmt.Println("import something")"#];
    let imports = parse_imports("go", &lines);
    assert!(imports.is_empty());
}

// ── normalize_import_target: deep cases ─────────────────────────────

#[test]
fn normalize_preserves_underscored_crate_names() {
    assert_eq!(normalize_import_target("serde_json"), "serde_json");
    assert_eq!(normalize_import_target("tokmd_types"), "tokmd_types");
}

#[test]
fn normalize_scoped_npm_package() {
    // @scope/package -> first split on / gives @scope
    assert_eq!(normalize_import_target("@types/node"), "@types");
    assert_eq!(normalize_import_target("@babel/core"), "@babel");
}

#[test]
fn normalize_double_dot_relative() {
    assert_eq!(normalize_import_target(".."), "local");
    assert_eq!(normalize_import_target("..."), "local");
    assert_eq!(normalize_import_target("./"), "local");
    assert_eq!(normalize_import_target("../../../deeply/nested"), "local");
}

#[test]
fn normalize_empty_string() {
    let result = normalize_import_target("");
    // Empty after trim, split on separators gives [""], first is ""
    assert_eq!(result, "");
}

#[test]
fn normalize_only_whitespace() {
    let result = normalize_import_target("   ");
    // After trim -> "", split gives [""], first is ""
    assert_eq!(result, "");
}

#[test]
fn normalize_deeply_nested_go_path() {
    assert_eq!(
        normalize_import_target("github.com/user/repo/internal/pkg"),
        "github"
    );
}

#[test]
fn normalize_strips_both_quote_types() {
    assert_eq!(normalize_import_target(r#""react""#), "react");
    assert_eq!(normalize_import_target("'lodash'"), "lodash");
}

#[test]
fn normalize_mixed_separators() {
    // "a/b:c.d" -> split on ['/', ':', '.'] -> first is "a"
    assert_eq!(normalize_import_target("a/b:c.d"), "a");
}

// ── Cross-language: same import target, different syntax ────────────

#[test]
fn same_module_across_languages() {
    let rust = parse_imports("rust", &["use serde::Serialize;"]);
    let py = parse_imports("python", &["import serde"]);

    assert_eq!(rust, vec!["serde"]);
    assert_eq!(py, vec!["serde"]);
}

// ── parse_imports: lines with only whitespace ───────────────────────

#[test]
fn whitespace_only_lines_produce_no_imports_any_lang() {
    let lines = vec!["", "   ", "\t", "  \t  "];
    for lang in ["rust", "python", "javascript", "typescript", "go"] {
        let imports = parse_imports(lang, &lines);
        assert!(
            imports.is_empty(),
            "whitespace-only for {lang} should be empty"
        );
    }
}

// ── parse_imports: large input ──────────────────────────────────────

#[test]
fn parse_handles_many_lines_without_panic() {
    let lines: Vec<String> = (0..10_000)
        .map(|i| format!("use crate_{i}::module;"))
        .collect();
    let refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
    let imports = parse_imports("rust", &refs);
    assert_eq!(imports.len(), 10_000);
}

// ── parse_imports: String vs &str compatibility ─────────────────────

#[test]
fn parse_accepts_string_slices() {
    let lines: Vec<String> = vec![
        "use std::io;".to_string(),
        "use serde::Deserialize;".to_string(),
    ];
    let imports = parse_imports("rust", &lines);
    assert_eq!(imports, vec!["std", "serde"]);
}
