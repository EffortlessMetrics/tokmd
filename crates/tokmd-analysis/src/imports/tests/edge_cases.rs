//! Edge-case tests for import parsing and normalization.

use crate::imports::{normalize_import_target, parse_imports};

// ── Circular / self-referencing import targets ─────────────────────

#[test]
fn rust_self_use_produces_self_root() {
    let lines = ["use self::inner::Foo;"];
    let imports = parse_imports("rust", &lines);
    assert_eq!(imports, vec!["self"]);
}

#[test]
fn rust_super_use_produces_super_root() {
    let lines = ["use super::sibling::Bar;"];
    let imports = parse_imports("rust", &lines);
    assert_eq!(imports, vec!["super"]);
}

#[test]
fn python_self_import_from_dot() {
    // `from . import X` — the module name is just "."
    let lines = ["from . import something"];
    let imports = parse_imports("python", &lines);
    assert_eq!(imports, vec!["."]);
    // Normalization treats dot-prefixed as local
    assert_eq!(normalize_import_target(&imports[0]), "local");
}

#[test]
fn python_deeply_relative_import() {
    let lines = ["from ...deeply.nested import thing"];
    let imports = parse_imports("python", &lines);
    assert_eq!(imports, vec!["...deeply.nested"]);
    assert_eq!(normalize_import_target(&imports[0]), "local");
}

// ── Empty / degenerate file contents ───────────────────────────────

#[test]
fn completely_empty_input_for_all_languages() {
    let empty: Vec<&str> = vec![];
    for lang in [
        "rust",
        "python",
        "javascript",
        "typescript",
        "go",
        "c",
        "java",
    ] {
        assert!(parse_imports(lang, &empty).is_empty());
    }
}

#[test]
fn file_with_only_comments_produces_no_imports() {
    let rust_lines = ["// use std::io;", "/* use serde::Serialize; */"];
    assert!(parse_imports("rust", &rust_lines).is_empty());

    let py_lines = ["# import os", "# from pathlib import Path"];
    assert!(parse_imports("python", &py_lines).is_empty());

    let js_lines = ["// import React from 'react';", "/* no require here */"];
    assert!(parse_imports("javascript", &js_lines).is_empty());

    let go_lines = ["// import \"fmt\"", "/* import \"os\" */"];
    assert!(parse_imports("go", &go_lines).is_empty());
}

#[test]
fn file_with_only_newlines_and_tabs() {
    let lines = ["\n", "\t", "\r\n", "", "  \t  "];
    for lang in ["rust", "python", "javascript", "go"] {
        assert!(parse_imports(lang, &lines).is_empty());
    }
}

// ── Deeply nested import paths ─────────────────────────────────────

#[test]
fn rust_deeply_nested_module_path() {
    let lines = ["use a::b::c::d::e::f::g::h::i::j;"];
    let imports = parse_imports("rust", &lines);
    assert_eq!(imports, vec!["a"]);
}

#[test]
fn python_deeply_nested_dotted_module() {
    let lines = ["import a.b.c.d.e.f.g.h"];
    let imports = parse_imports("python", &lines);
    assert_eq!(imports, vec!["a.b.c.d.e.f.g.h"]);
    assert_eq!(normalize_import_target(&imports[0]), "a");
}

#[test]
fn go_deeply_nested_module_path() {
    let lines = [r#"import "github.com/org/repo/internal/pkg/sub/v2/client""#];
    let imports = parse_imports("go", &lines);
    assert_eq!(
        imports,
        vec!["github.com/org/repo/internal/pkg/sub/v2/client"]
    );
    assert_eq!(normalize_import_target(&imports[0]), "github");
}

#[test]
fn js_deeply_nested_relative_path() {
    let lines = [r#"import x from "../../../../shared/utils/helpers";"#];
    let imports = parse_imports("javascript", &lines);
    assert_eq!(imports, vec!["../../../../shared/utils/helpers"]);
    assert_eq!(normalize_import_target(&imports[0]), "local");
}

// ── Malformed / unusual import statements ──────────────────────────

#[test]
fn rust_use_without_semicolon_still_extracts() {
    // The parser strips `;` from end, but it also works without one
    let lines = ["use std::io"];
    let imports = parse_imports("rust", &lines);
    assert_eq!(imports, vec!["std"]);
}

#[test]
fn rust_mod_with_braces_extracts_name_with_brace() {
    let lines = ["mod inline {"];
    let imports = parse_imports("rust", &lines);
    assert_eq!(imports.len(), 1);
    // The parser captures "inline {" since it doesn't handle inline mod blocks specially
    assert!(imports[0].starts_with("inline"));
}

#[test]
fn python_import_with_trailing_comment() {
    let lines = ["import os  # standard library"];
    let imports = parse_imports("python", &lines);
    // split_whitespace gives "os" as first token
    assert_eq!(imports, vec!["os"]);
}

#[test]
fn go_import_block_with_blank_lines() {
    let lines = ["import (", "", r#"    "fmt""#, "", r#"    "os""#, "", ")"];
    let imports = parse_imports("go", &lines);
    assert_eq!(imports, vec!["fmt", "os"]);
}

#[test]
fn go_unclosed_import_block_extracts_what_it_can() {
    // If the closing paren is missing, the parser stays in block mode
    let lines = [
        "import (",
        r#"    "fmt""#,
        r#"    "os""#,
        // no closing ")"
    ];
    let imports = parse_imports("go", &lines);
    assert_eq!(imports, vec!["fmt", "os"]);
}

#[test]
fn js_import_with_empty_from_clause() {
    // import {} from "react" — still has a quoted target
    let lines = [r#"import {} from "react";"#];
    let imports = parse_imports("javascript", &lines);
    assert_eq!(imports, vec!["react"]);
}

#[test]
fn js_require_with_template_literal_not_captured() {
    // Template literals use backticks, not quotes
    let lines = ["const x = require(`./dynamic`);"];
    let imports = parse_imports("javascript", &lines);
    // extract_quoted looks for ' or " — backticks are not matched
    assert!(imports.is_empty());
}

// ── Unicode and special characters ─────────────────────────────────

#[test]
fn unicode_in_non_import_lines_does_not_crash() {
    let lines = ["// 日本語コメント", "use std::io;", "// émojis 🦀🐍"];
    let imports = parse_imports("rust", &lines);
    assert_eq!(imports, vec!["std"]);
}

#[test]
fn python_import_with_unicode_module_name() {
    let lines = ["import café"];
    let imports = parse_imports("python", &lines);
    assert_eq!(imports, vec!["café"]);
}

// ── Normalization edge cases ───────────────────────────────────────

#[test]
fn normalize_single_dot_is_local() {
    assert_eq!(normalize_import_target("."), "local");
}

#[test]
fn normalize_triple_dot_is_local() {
    assert_eq!(normalize_import_target("..."), "local");
}

#[test]
fn normalize_target_with_only_separators() {
    // "/" splits to ["", ""] -> first is ""
    assert_eq!(normalize_import_target("/"), "");
    // ":" splits to ["", ""] -> first is ""
    assert_eq!(normalize_import_target(":"), "");
}

#[test]
fn normalize_target_with_mixed_quotes_strips_outermost() {
    // Double quotes wrapping
    assert_eq!(normalize_import_target(r#""foo/bar""#), "foo");
    // Single quotes wrapping
    assert_eq!(normalize_import_target("'baz.qux'"), "baz");
}

#[test]
fn normalize_preserves_hyphens_in_package_names() {
    assert_eq!(normalize_import_target("my-package/utils"), "my-package");
    assert_eq!(normalize_import_target("@scope/my-lib"), "@scope");
}

#[test]
fn normalize_very_long_target() {
    let long_path = (0..100)
        .map(|i| format!("seg{i}"))
        .collect::<Vec<_>>()
        .join("/");
    let result = normalize_import_target(&long_path);
    assert_eq!(result, "seg0");
}

// ── Interaction between parse and normalize ─────────────────────────

#[test]
fn parse_then_normalize_pipeline_for_rust() {
    let lines = [
        "use std::collections::HashMap;",
        "use crate::config::Settings;",
        "use super::parent;",
        "mod child;",
    ];
    let imports = parse_imports("rust", &lines);
    let normalized: Vec<String> = imports.iter().map(|t| normalize_import_target(t)).collect();
    assert_eq!(normalized, vec!["std", "crate", "super", "child"]);
}

#[test]
fn parse_then_normalize_pipeline_for_js() {
    let lines = [
        r#"import React from "react";"#,
        r#"import utils from "./utils";"#,
        r#"const fs = require("fs/promises");"#,
        r#"const cfg = require("../config");"#,
    ];
    let imports = parse_imports("javascript", &lines);
    let normalized: Vec<String> = imports.iter().map(|t| normalize_import_target(t)).collect();
    assert_eq!(normalized, vec!["react", "local", "fs", "local"]);
}

#[test]
fn parse_then_normalize_pipeline_for_python() {
    let lines = [
        "import os.path",
        "from . import utils",
        "from collections import OrderedDict",
        "import numpy",
    ];
    let imports = parse_imports("python", &lines);
    let normalized: Vec<String> = imports.iter().map(|t| normalize_import_target(t)).collect();
    assert_eq!(normalized, vec!["os", "local", "collections", "numpy"]);
}

#[test]
fn parse_then_normalize_pipeline_for_go() {
    let lines = [
        "import (",
        r#""fmt""#,
        r#""github.com/user/repo/pkg""#,
        r#""net/http""#,
        ")",
    ];
    let imports = parse_imports("go", &lines);
    let normalized: Vec<String> = imports.iter().map(|t| normalize_import_target(t)).collect();
    assert_eq!(normalized, vec!["fmt", "github", "net"]);
}

// ── Multiple import blocks in a single file ────────────────────────

#[test]
fn go_multiple_import_blocks_all_captured() {
    let lines = [
        "import (",
        r#""fmt""#,
        ")",
        "// some code",
        "func init() {}",
        "import (",
        r#""os""#,
        r#""io""#,
        ")",
    ];
    let imports = parse_imports("go", &lines);
    assert_eq!(imports, vec!["fmt", "os", "io"]);
}

#[test]
fn rust_imports_scattered_through_file() {
    let lines = [
        "use std::io;",
        "",
        "fn helper() {}",
        "",
        "use serde::Serialize;",
        "",
        "fn main() {}",
        "",
        "use anyhow::Result;",
    ];
    let imports = parse_imports("rust", &lines);
    assert_eq!(imports, vec!["std", "serde", "anyhow"]);
}

#[test]
fn python_imports_at_top_and_inside_function() {
    let lines = [
        "import os",
        "",
        "def foo():",
        "    import sys",
        "    from pathlib import Path",
    ];
    let imports = parse_imports("python", &lines);
    assert_eq!(imports, vec!["os", "sys", "pathlib"]);
}
