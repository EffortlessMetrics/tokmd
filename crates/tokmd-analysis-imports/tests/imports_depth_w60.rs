//! Wave-60 depth tests for `tokmd-analysis-imports`.
//!
//! Covers:
//! - BDD-style tests for import graph construction edge cases
//! - Property tests for import parsing determinism
//! - Tests for all supported languages (Rust, Python, JS/TS, Go)
//! - Circular imports, deep nesting, re-exports
//! - Empty/degenerate inputs, large inputs, special characters

use std::collections::{BTreeMap, BTreeSet};

use proptest::prelude::*;
use tokmd_analysis_imports::{normalize_import_target, parse_imports, supports_language};

// ── Helpers ─────────────────────────────────────────────────────────

fn build_graph(files: &[(&str, &str, &[&str])]) -> BTreeMap<String, BTreeSet<String>> {
    let mut graph: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for &(file, lang, lines) in files {
        let raw = parse_imports(lang, lines);
        let targets: BTreeSet<String> = raw.iter().map(|t| normalize_import_target(t)).collect();
        graph.insert(file.to_string(), targets);
    }
    graph
}

fn has_cycle(graph: &BTreeMap<String, BTreeSet<String>>) -> bool {
    let mut visited = BTreeSet::new();
    let mut stack = BTreeSet::new();
    for node in graph.keys() {
        if dfs_cycle(node, graph, &mut visited, &mut stack) {
            return true;
        }
    }
    false
}

fn dfs_cycle(
    node: &str,
    graph: &BTreeMap<String, BTreeSet<String>>,
    visited: &mut BTreeSet<String>,
    stack: &mut BTreeSet<String>,
) -> bool {
    if stack.contains(node) {
        return true;
    }
    if visited.contains(node) {
        return false;
    }
    visited.insert(node.to_string());
    stack.insert(node.to_string());
    if let Some(deps) = graph.get(node) {
        for dep in deps {
            if dfs_cycle(dep, graph, visited, stack) {
                return true;
            }
        }
    }
    stack.remove(node);
    false
}

fn group_by_root(lang: &str, lines: &[&str]) -> BTreeMap<String, usize> {
    let imports = parse_imports(lang, lines);
    let mut groups: BTreeMap<String, usize> = BTreeMap::new();
    for target in &imports {
        let root = normalize_import_target(target);
        *groups.entry(root).or_insert(0) += 1;
    }
    groups
}

// =============================================================================
// 1. BDD: Import graph construction edge cases
// =============================================================================

// ── Scenario: fan-in pattern (multiple files import same dependency) ─

#[test]
fn given_fan_in_pattern_when_building_graph_then_shared_dep_appears_in_all() {
    let files: Vec<(&str, &str, &[&str])> = vec![
        ("a.rs", "rust", &["use shared::Api;"]),
        ("b.rs", "rust", &["use shared::Config;"]),
        ("c.rs", "rust", &["use shared::Types;"]),
        ("shared.rs", "rust", &[]),
    ];
    let graph = build_graph(&files);
    for name in ["a.rs", "b.rs", "c.rs"] {
        assert!(
            graph[name].contains("shared"),
            "{name} should depend on shared"
        );
    }
    assert!(graph["shared.rs"].is_empty());
}

// ── Scenario: fan-out pattern (one file imports many) ───────────────

#[test]
fn given_fan_out_pattern_when_building_graph_then_hub_has_all_deps() {
    let lines: Vec<String> = (0..30).map(|i| format!("import dep_{i}")).collect();
    let refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
    let files: Vec<(&str, &str, &[&str])> = vec![("hub.py", "python", &refs)];
    let graph = build_graph(&files);
    assert_eq!(graph["hub.py"].len(), 30);
}

// ── Scenario: disconnected components in graph ──────────────────────

#[test]
fn given_disconnected_components_when_building_graph_then_no_cross_edges() {
    let files: Vec<(&str, &str, &[&str])> = vec![
        ("a.rs", "rust", &["use b::X;"]),
        ("b.rs", "rust", &[]),
        ("c.py", "python", &["import d"]),
        ("d.py", "python", &[]),
    ];
    let graph = build_graph(&files);
    // Rust component: a->b
    assert!(graph["a.rs"].contains("b"));
    assert!(graph["b.rs"].is_empty());
    // Python component: c->d
    assert!(graph["c.py"].contains("d"));
    assert!(graph["d.py"].is_empty());
    // No cross-component edges
    assert!(!graph["a.rs"].contains("d"));
    assert!(!graph["c.py"].contains("b"));
}

// ── Scenario: complete graph (every node imports every other) ───────

#[test]
fn given_complete_graph_pattern_when_building_then_all_cross_edges_exist() {
    let files: Vec<(&str, &str, &[&str])> = vec![
        ("a.py", "python", &["import b", "import c"]),
        ("b.py", "python", &["import a", "import c"]),
        ("c.py", "python", &["import a", "import b"]),
    ];
    let graph = build_graph(&files);
    assert!(graph["a.py"].contains("b") && graph["a.py"].contains("c"));
    assert!(graph["b.py"].contains("a") && graph["b.py"].contains("c"));
    assert!(graph["c.py"].contains("a") && graph["c.py"].contains("b"));
}

// ── Scenario: long chain cycle detection ────────────────────────────

#[test]
fn given_long_chain_with_back_edge_when_checking_cycle_then_detected() {
    let mut graph: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for i in 0..50 {
        graph.insert(format!("n{i}"), BTreeSet::from([format!("n{}", i + 1)]));
    }
    graph.insert("n50".to_string(), BTreeSet::from(["n0".to_string()]));
    assert!(has_cycle(&graph));
}

#[test]
fn given_long_chain_without_back_edge_when_checking_cycle_then_not_detected() {
    let mut graph: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for i in 0..50 {
        graph.insert(format!("n{i}"), BTreeSet::from([format!("n{}", i + 1)]));
    }
    graph.insert("n50".to_string(), BTreeSet::new());
    assert!(!has_cycle(&graph));
}

// ── Scenario: diamond with extra edges (no cycle) ───────────────────

#[test]
fn given_diamond_with_shortcut_edge_when_checking_cycle_then_not_detected() {
    let mut graph: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    graph.insert(
        "a".into(),
        BTreeSet::from(["b".into(), "c".into(), "d".into()]),
    );
    graph.insert("b".into(), BTreeSet::from(["d".into()]));
    graph.insert("c".into(), BTreeSet::from(["d".into()]));
    graph.insert("d".into(), BTreeSet::new());
    assert!(!has_cycle(&graph));
}

// =============================================================================
// 2. Language-specific parsing: Rust edge cases
// =============================================================================

#[test]
fn rust_use_with_rename_as_extracts_root() {
    let lines = ["use std::io::Error as IoError;"];
    let imports = parse_imports("rust", &lines);
    assert_eq!(imports, vec!["std"]);
}

#[test]
fn rust_use_glob_star_extracts_root() {
    let lines = ["use std::prelude::v1::*;"];
    let imports = parse_imports("rust", &lines);
    assert_eq!(imports, vec!["std"]);
}

#[test]
fn rust_use_with_deeply_nested_braces() {
    let lines = ["use std::{collections::{BTreeMap, HashMap}, io::{self, Read, Write}};"];
    let imports = parse_imports("rust", &lines);
    assert_eq!(imports, vec!["std"]);
}

#[test]
fn rust_multiple_mod_declarations_preserve_order() {
    let lines = ["mod z_mod;", "mod a_mod;", "mod m_mod;"];
    let imports = parse_imports("rust", &lines);
    assert_eq!(imports, vec!["z_mod", "a_mod", "m_mod"]);
}

#[test]
fn rust_use_crate_and_super_and_self_all_captured() {
    let lines = [
        "use crate::lib::Foo;",
        "use super::parent::Bar;",
        "use self::inner::Baz;",
    ];
    let imports = parse_imports("rust", &lines);
    assert_eq!(imports, vec!["crate", "super", "self"]);
}

#[test]
fn rust_use_without_semicolon_still_captures() {
    let lines = ["use serde::Serialize"];
    let imports = parse_imports("rust", &lines);
    assert_eq!(imports, vec!["serde"]);
}

#[test]
fn rust_pub_use_is_not_captured() {
    let lines = ["pub use crate::api::Handler;"];
    assert!(parse_imports("rust", &lines).is_empty());
}

#[test]
fn rust_pub_mod_is_not_captured() {
    let lines = ["pub mod routes;"];
    assert!(parse_imports("rust", &lines).is_empty());
}

// =============================================================================
// 3. Language-specific parsing: Python edge cases
// =============================================================================

#[test]
fn python_from_import_with_parentheses_multiline_first_line_only() {
    let lines = ["from collections import (", "    OrderedDict,", "    defaultdict,", ")"];
    let imports = parse_imports("python", &lines);
    // Only the "from collections" line matches
    assert_eq!(imports, vec!["collections"]);
}

#[test]
fn python_import_comma_separated_only_first_is_captured() {
    // "import os, sys" — first whitespace token after "import " is "os,"
    let lines = ["import os, sys"];
    let imports = parse_imports("python", &lines);
    assert_eq!(imports, vec!["os,"]);
}

#[test]
fn python_deeply_nested_from_relative() {
    let lines = ["from .....very.deep.module import something"];
    let imports = parse_imports("python", &lines);
    assert_eq!(imports, vec![".....very.deep.module"]);
    assert_eq!(normalize_import_target(&imports[0]), "local");
}

#[test]
fn python_conditional_import_in_try_block() {
    let lines = [
        "try:",
        "    import ujson as json",
        "except ImportError:",
        "    import json",
    ];
    let imports = parse_imports("python", &lines);
    assert_eq!(imports, vec!["ujson", "json"]);
}

#[test]
fn python_import_with_backslash_continuation_only_captures_first_line() {
    let lines = ["import \\", "    os"];
    let imports = parse_imports("python", &lines);
    // "import \" -> first whitespace token is "\"
    assert_eq!(imports.len(), 1);
}

#[test]
fn python_from_future_import() {
    let lines = ["from __future__ import annotations"];
    let imports = parse_imports("python", &lines);
    assert_eq!(imports, vec!["__future__"]);
}

// =============================================================================
// 4. Language-specific parsing: JavaScript/TypeScript edge cases
// =============================================================================

#[test]
fn js_side_effect_import_double_quotes() {
    let lines = [r#"import "core-js/stable";"#];
    let imports = parse_imports("javascript", &lines);
    assert_eq!(imports, vec!["core-js/stable"]);
}

#[test]
fn js_side_effect_import_single_quotes() {
    let lines = ["import 'regenerator-runtime/runtime';"];
    let imports = parse_imports("javascript", &lines);
    assert_eq!(imports, vec!["regenerator-runtime/runtime"]);
}

#[test]
fn js_import_and_require_on_same_line_both_captured() {
    let lines = [r#"import x from "foo"; const y = require("bar");"#];
    let imports = parse_imports("javascript", &lines);
    assert_eq!(imports, vec!["foo", "bar"]);
}

#[test]
fn js_require_with_no_quotes_not_captured() {
    let lines = ["const x = require(variable);"];
    let imports = parse_imports("javascript", &lines);
    assert!(imports.is_empty());
}

#[test]
fn js_import_with_template_literal_not_captured() {
    let lines = ["import x from `template`;"];
    let imports = parse_imports("javascript", &lines);
    // Backticks are not recognized as quotes by extract_quoted
    assert!(imports.is_empty());
}

#[test]
fn ts_import_type_with_curly_braces() {
    let lines = [r#"import type { FC, ReactNode } from "react";"#];
    let imports = parse_imports("typescript", &lines);
    assert_eq!(imports, vec!["react"]);
}

#[test]
fn ts_and_js_parse_identically() {
    let lines = [
        r#"import React from "react";"#,
        r#"const fs = require("fs");"#,
    ];
    let js = parse_imports("javascript", &lines);
    let ts = parse_imports("typescript", &lines);
    assert_eq!(js, ts);
}

#[test]
fn js_empty_string_require_not_captured() {
    let lines = [r#"const x = require("");"#];
    let imports = parse_imports("javascript", &lines);
    assert!(imports.is_empty());
}

// =============================================================================
// 5. Language-specific parsing: Go edge cases
// =============================================================================

#[test]
fn go_block_import_with_blank_identifier() {
    let lines = ["import (", r#"    _ "image/png""#, r#"    "fmt""#, ")"];
    let imports = parse_imports("go", &lines);
    assert_eq!(imports, vec!["image/png", "fmt"]);
}

#[test]
fn go_block_import_with_dot_import() {
    let lines = ["import (", r#"    . "testing""#, ")"];
    let imports = parse_imports("go", &lines);
    assert_eq!(imports, vec!["testing"]);
}

#[test]
fn go_multiple_separate_blocks_all_captured() {
    let lines = [
        "import (",
        r#""fmt""#,
        ")",
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
fn go_block_with_comment_lines_only_extracts_quoted() {
    let lines = [
        "import (",
        "    // stdlib",
        r#"    "fmt""#,
        "    // external",
        r#"    "github.com/pkg/errors""#,
        ")",
    ];
    let imports = parse_imports("go", &lines);
    assert_eq!(imports, vec!["fmt", "github.com/pkg/errors"]);
}

#[test]
fn go_single_import_with_alias() {
    let lines = [r#"import log "github.com/sirupsen/logrus""#];
    let imports = parse_imports("go", &lines);
    assert_eq!(imports, vec!["github.com/sirupsen/logrus"]);
}

#[test]
fn go_unclosed_block_still_extracts() {
    let lines = ["import (", r#""fmt""#, r#""os""#];
    let imports = parse_imports("go", &lines);
    assert_eq!(imports, vec!["fmt", "os"]);
}

// =============================================================================
// 6. Normalization edge cases
// =============================================================================

#[test]
fn normalize_empty_string_returns_empty() {
    assert_eq!(normalize_import_target(""), "");
}

#[test]
fn normalize_whitespace_only_returns_empty() {
    assert_eq!(normalize_import_target("   "), "");
    assert_eq!(normalize_import_target("\t"), "");
}

#[test]
fn normalize_single_dot_is_local() {
    assert_eq!(normalize_import_target("."), "local");
}

#[test]
fn normalize_many_dots_is_local() {
    assert_eq!(normalize_import_target("...."), "local");
    assert_eq!(normalize_import_target("......."), "local");
}

#[test]
fn normalize_dot_slash_prefix_is_local() {
    assert_eq!(normalize_import_target("./foo/bar"), "local");
    assert_eq!(normalize_import_target("../baz"), "local");
}

#[test]
fn normalize_strips_double_quotes() {
    assert_eq!(normalize_import_target(r#""react""#), "react");
}

#[test]
fn normalize_strips_single_quotes() {
    assert_eq!(normalize_import_target("'lodash'"), "lodash");
}

#[test]
fn normalize_at_scope_returns_scope() {
    assert_eq!(normalize_import_target("@types/node"), "@types");
    assert_eq!(normalize_import_target("@babel/core/lib"), "@babel");
}

#[test]
fn normalize_rust_colon_path_returns_first_segment() {
    assert_eq!(normalize_import_target("serde::Serialize"), "serde");
    assert_eq!(
        normalize_import_target("tokmd_types::Receipt"),
        "tokmd_types"
    );
}

#[test]
fn normalize_go_dotted_path_returns_first_segment() {
    assert_eq!(
        normalize_import_target("github.com/user/repo"),
        "github"
    );
}

#[test]
fn normalize_slash_path_returns_first_segment() {
    assert_eq!(normalize_import_target("fs/promises"), "fs");
    assert_eq!(normalize_import_target("net/http"), "net");
}

#[test]
fn normalize_leading_slash_returns_empty_first_segment() {
    assert_eq!(normalize_import_target("/absolute/path"), "");
}

#[test]
fn normalize_preserves_underscores_and_hyphens_in_first_segment() {
    assert_eq!(normalize_import_target("my_crate::Foo"), "my_crate");
    assert_eq!(normalize_import_target("my-pkg/utils"), "my-pkg");
}

#[test]
fn normalize_is_idempotent_for_simple_names() {
    for name in ["std", "os", "react", "serde", "fmt", "lodash"] {
        let first = normalize_import_target(name);
        let second = normalize_import_target(&first);
        assert_eq!(first, second, "idempotent check failed for {name}");
    }
}

// =============================================================================
// 7. Cross-language consistency
// =============================================================================

#[test]
fn same_module_name_normalizes_identically_across_all_languages() {
    let rust = parse_imports("rust", &["use serde::Serialize;"]);
    let py = parse_imports("python", &["import serde"]);
    assert_eq!(
        normalize_import_target(&rust[0]),
        normalize_import_target(&py[0])
    );
}

#[test]
fn relative_imports_normalize_to_local_in_all_languages() {
    let js = parse_imports("javascript", &[r#"import x from "./foo";"#]);
    let py = parse_imports("python", &["from . import foo"]);

    assert_eq!(normalize_import_target(&js[0]), "local");
    assert_eq!(normalize_import_target(&py[0]), "local");
}

#[test]
fn parse_imports_empty_lines_returns_empty_for_all_languages() {
    let empty: Vec<&str> = vec![];
    for lang in ["rust", "python", "javascript", "typescript", "go"] {
        assert!(
            parse_imports(lang, &empty).is_empty(),
            "empty for {lang} should be empty"
        );
    }
}

#[test]
fn parse_imports_blank_lines_returns_empty_for_all_languages() {
    let blanks = vec!["", "   ", "\t"];
    for lang in ["rust", "python", "javascript", "typescript", "go"] {
        assert!(
            parse_imports(lang, &blanks).is_empty(),
            "blanks for {lang} should be empty"
        );
    }
}

// =============================================================================
// 8. supports_language coverage
// =============================================================================

#[test]
fn supports_language_accepts_mixed_case() {
    assert!(supports_language("RuSt"));
    assert!(supports_language("JAVASCRIPT"));
    assert!(supports_language("gO"));
    assert!(supports_language("pYtHoN"));
    assert!(supports_language("TypeScript"));
}

#[test]
fn supports_language_rejects_abbreviations_and_variants() {
    assert!(!supports_language("rs"));
    assert!(!supports_language("js"));
    assert!(!supports_language("ts"));
    assert!(!supports_language("py"));
    assert!(!supports_language("golang"));
    assert!(!supports_language("python3"));
    assert!(!supports_language("node"));
    assert!(!supports_language("ecmascript"));
}

#[test]
fn supports_language_rejects_empty_and_whitespace() {
    assert!(!supports_language(""));
    assert!(!supports_language(" "));
    assert!(!supports_language("\t"));
}

#[test]
fn unsupported_languages_always_return_empty_imports() {
    let lines = ["#include <stdio.h>", "import java.util.*;", "require 'rails'"];
    for lang in ["c", "c++", "java", "ruby", "kotlin", "swift", "haskell", ""] {
        assert!(
            parse_imports(lang, &lines).is_empty(),
            "expected empty for unsupported lang '{lang}'"
        );
    }
}

// =============================================================================
// 9. Module grouping / deduplication
// =============================================================================

#[test]
fn grouping_rust_deduplicates_same_crate() {
    let lines: &[&str] = &[
        "use std::io;",
        "use std::fs;",
        "use std::collections::HashMap;",
        "use serde::Serialize;",
    ];
    let groups = group_by_root("rust", lines);
    assert_eq!(groups["std"], 3);
    assert_eq!(groups["serde"], 1);
}

#[test]
fn grouping_js_relative_all_collapse_to_local() {
    let lines: &[&str] = &[
        r#"import a from "./a";"#,
        r#"import b from "../b";"#,
        r#"import c from "./c/d";"#,
        r#"import React from "react";"#,
    ];
    let groups = group_by_root("javascript", lines);
    assert_eq!(groups["local"], 3);
    assert_eq!(groups["react"], 1);
}

#[test]
fn grouping_go_external_all_collapse_to_domain() {
    let lines: &[&str] = &[
        "import (",
        r#""github.com/user/repo1""#,
        r#""github.com/user/repo2""#,
        r#""gitlab.com/team/project""#,
        r#""fmt""#,
        ")",
    ];
    let groups = group_by_root("go", lines);
    assert_eq!(groups["github"], 2);
    assert_eq!(groups["gitlab"], 1);
    assert_eq!(groups["fmt"], 1);
}

// =============================================================================
// 10. Unicode and special characters
// =============================================================================

#[test]
fn unicode_in_comments_does_not_crash_any_parser() {
    let lines = ["// 日本語コメント 🦀", "use std::io;", "// émojis 🐍"];
    let imports = parse_imports("rust", &lines);
    assert_eq!(imports, vec!["std"]);
}

#[test]
fn python_import_unicode_module_name() {
    let lines = ["import café"];
    let imports = parse_imports("python", &lines);
    assert_eq!(imports, vec!["café"]);
}

#[test]
fn unicode_in_go_comments_does_not_crash() {
    let lines = ["import (", "// 注释", r#""fmt""#, ")"];
    let imports = parse_imports("go", &lines);
    assert_eq!(imports, vec!["fmt"]);
}

// =============================================================================
// 11. Large / stress inputs
// =============================================================================

#[test]
fn parse_2000_python_imports_without_panic() {
    let lines: Vec<String> = (0..2000).map(|i| format!("import mod_{i}")).collect();
    let refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
    let imports = parse_imports("python", &refs);
    assert_eq!(imports.len(), 2000);
}

#[test]
fn parse_go_block_with_500_imports() {
    let mut lines = vec!["import (".to_string()];
    for i in 0..500 {
        lines.push(format!(r#""pkg_{i}""#));
    }
    lines.push(")".to_string());
    let refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
    let imports = parse_imports("go", &refs);
    assert_eq!(imports.len(), 500);
}

#[test]
fn parse_rust_10000_use_statements() {
    let lines: Vec<String> = (0..10_000)
        .map(|i| format!("use crate_{i}::module;"))
        .collect();
    let refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
    let imports = parse_imports("rust", &refs);
    assert_eq!(imports.len(), 10_000);
}

#[test]
fn normalize_very_long_slash_path() {
    let long_path = (0..1000)
        .map(|i| format!("seg{i}"))
        .collect::<Vec<_>>()
        .join("/");
    assert_eq!(normalize_import_target(&long_path), "seg0");
}

// =============================================================================
// 12. Parse then normalize pipeline
// =============================================================================

#[test]
fn pipeline_rust_mixed_imports() {
    let lines = [
        "use std::io;",
        "use crate::config;",
        "use super::parent;",
        "mod child;",
    ];
    let imports = parse_imports("rust", &lines);
    let normalized: Vec<String> = imports.iter().map(|t| normalize_import_target(t)).collect();
    assert_eq!(normalized, vec!["std", "crate", "super", "child"]);
}

#[test]
fn pipeline_python_mixed_imports() {
    let lines = [
        "import os",
        "from . import utils",
        "from collections import OrderedDict",
        "import numpy",
    ];
    let imports = parse_imports("python", &lines);
    let normalized: Vec<String> = imports.iter().map(|t| normalize_import_target(t)).collect();
    assert_eq!(normalized, vec!["os", "local", "collections", "numpy"]);
}

#[test]
fn pipeline_js_mixed_imports() {
    let lines = [
        r#"import React from "react";"#,
        r#"import utils from "./utils";"#,
        r#"const fs = require("fs/promises");"#,
    ];
    let imports = parse_imports("javascript", &lines);
    let normalized: Vec<String> = imports.iter().map(|t| normalize_import_target(t)).collect();
    assert_eq!(normalized, vec!["react", "local", "fs"]);
}

#[test]
fn pipeline_go_mixed_imports() {
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

// =============================================================================
// 13. Deterministic ordering
// =============================================================================

#[test]
fn parse_preserves_source_order_for_all_languages() {
    let rust_lines = ["use z::A;", "use a::B;", "use m::C;"];
    assert_eq!(parse_imports("rust", &rust_lines), vec!["z", "a", "m"]);

    let py_lines = ["import z_mod", "import a_mod", "import m_mod"];
    assert_eq!(
        parse_imports("python", &py_lines),
        vec!["z_mod", "a_mod", "m_mod"]
    );
}

#[test]
fn parse_is_deterministic_across_500_calls() {
    let lines = ["use std::io;", "use serde::Serialize;", "use anyhow::Result;"];
    let baseline = parse_imports("rust", &lines);
    for _ in 0..500 {
        assert_eq!(parse_imports("rust", &lines), baseline);
    }
}

#[test]
fn graph_keys_are_btreemap_sorted() {
    let files: Vec<(&str, &str, &[&str])> = vec![
        ("z.rs", "rust", &["use a::X;"]),
        ("a.rs", "rust", &["use z::Y;"]),
        ("m.rs", "rust", &["use b::W;"]),
    ];
    let graph = build_graph(&files);
    let keys: Vec<&String> = graph.keys().collect();
    assert_eq!(keys, vec!["a.rs", "m.rs", "z.rs"]);
}

// =============================================================================
// 14. Property tests
// =============================================================================

fn arb_supported_lang() -> impl Strategy<Value = &'static str> {
    prop_oneof![
        Just("rust"),
        Just("javascript"),
        Just("typescript"),
        Just("python"),
        Just("go"),
    ]
}

proptest! {
    #[test]
    fn parse_imports_deterministic(
        lang in "[a-zA-Z]{0,16}",
        lines in prop::collection::vec("[ -~]{0,120}", 0..32)
    ) {
        let first = parse_imports(&lang, &lines);
        let second = parse_imports(&lang, &lines);
        prop_assert_eq!(first, second);
    }

    #[test]
    fn unsupported_lang_always_returns_empty(
        lang in "[a-zA-Z0-9_]{0,16}",
        lines in prop::collection::vec("[ -~]{0,60}", 0..16)
    ) {
        let lower = lang.to_ascii_lowercase();
        prop_assume!(!matches!(lower.as_str(), "rust" | "javascript" | "typescript" | "python" | "go"));
        prop_assert!(parse_imports(&lang, &lines).is_empty());
    }

    #[test]
    fn normalize_deterministic(target in "[a-zA-Z0-9_./:'\"-]{0,80}") {
        let first = normalize_import_target(&target);
        let second = normalize_import_target(&target);
        prop_assert_eq!(first, second);
    }

    #[test]
    fn relative_always_normalizes_to_local(suffix in "[a-zA-Z0-9_/-]{0,32}") {
        let target = format!(".{suffix}");
        prop_assert_eq!(normalize_import_target(&target), "local");
    }

    #[test]
    fn rust_use_always_produces_one_import(crate_name in "[a-z_][a-z0-9_]{0,15}") {
        let line = format!("use {crate_name}::Thing;");
        let imports = parse_imports("rust", &[line]);
        prop_assert_eq!(imports.len(), 1);
        prop_assert_eq!(&imports[0], &crate_name);
    }

    #[test]
    fn rust_mod_always_produces_one_import(mod_name in "[a-z_][a-z0-9_]{0,15}") {
        let line = format!("mod {mod_name};");
        let imports = parse_imports("rust", &[line]);
        prop_assert_eq!(imports.len(), 1);
        prop_assert_eq!(&imports[0], &mod_name);
    }

    #[test]
    fn python_import_always_produces_one_import(module in "[a-z][a-z0-9_]{0,15}") {
        let line = format!("import {module}");
        let imports = parse_imports("python", &[line]);
        prop_assert_eq!(imports.len(), 1);
        prop_assert_eq!(&imports[0], &module);
    }

    #[test]
    fn python_from_always_produces_one_import(module in "[a-z][a-z0-9_]{0,15}") {
        let line = format!("from {module} import thing");
        let imports = parse_imports("python", &[line]);
        prop_assert_eq!(imports.len(), 1);
        prop_assert_eq!(&imports[0], &module);
    }

    #[test]
    fn go_single_always_produces_one_import(pkg in "[a-z]{1,12}") {
        let line = format!(r#"import "{pkg}""#);
        let imports = parse_imports("go", &[line]);
        prop_assert_eq!(imports.len(), 1);
        prop_assert_eq!(&imports[0], &pkg);
    }

    #[test]
    fn go_block_count_matches_quoted_lines(pkgs in prop::collection::vec("[a-z]{1,8}", 1..10)) {
        let mut lines = vec!["import (".to_string()];
        for pkg in &pkgs {
            lines.push(format!(r#""{pkg}""#));
        }
        lines.push(")".to_string());
        let refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
        let imports = parse_imports("go", &refs);
        prop_assert_eq!(imports.len(), pkgs.len());
    }

    #[test]
    fn js_import_from_always_produces_one(pkg in "[a-z][a-z0-9-]{0,15}") {
        let line = format!(r#"import x from "{pkg}";"#);
        let imports = parse_imports("javascript", &[line]);
        prop_assert_eq!(imports.len(), 1);
        prop_assert_eq!(&imports[0], &pkg);
    }

    #[test]
    fn js_require_always_produces_one(pkg in "[a-z][a-z0-9-]{0,15}") {
        let line = format!(r#"const x = require("{pkg}");"#);
        let imports = parse_imports("javascript", &[line]);
        prop_assert_eq!(imports.len(), 1);
        prop_assert_eq!(&imports[0], &pkg);
    }

    #[test]
    fn ts_and_js_parse_identically_prop(
        lines in prop::collection::vec("[ -~]{0,100}", 0..16)
    ) {
        let js = parse_imports("javascript", &lines);
        let ts = parse_imports("typescript", &lines);
        prop_assert_eq!(js, ts);
    }

    #[test]
    fn output_count_le_input_lines(
        lang in arb_supported_lang(),
        lines in prop::collection::vec("[ -~]{0,100}", 0..32)
    ) {
        let imports = parse_imports(lang, &lines);
        prop_assert!(imports.len() <= lines.len());
    }

    #[test]
    fn normalize_idempotent_for_simple(name in "[a-z][a-z0-9_]{0,20}") {
        let first = normalize_import_target(&name);
        let second = normalize_import_target(&first);
        prop_assert_eq!(first, second);
    }

    #[test]
    fn normalize_never_returns_empty_for_alpha(target in "[a-zA-Z][a-zA-Z0-9_/-]{0,30}") {
        let result = normalize_import_target(&target);
        prop_assert!(!result.is_empty());
    }

    #[test]
    fn all_js_relative_imports_normalize_to_local(suffix in "[a-zA-Z0-9_/]{1,20}") {
        let line = format!(r#"import x from "./{suffix}";"#);
        let imports = parse_imports("javascript", &[line.as_str()]);
        prop_assert!(!imports.is_empty());
        let normalized = normalize_import_target(&imports[0]);
        prop_assert_eq!(normalized, "local");
    }

    #[test]
    fn supports_language_case_insensitive_prop(
        lang in arb_supported_lang(),
        upper in proptest::bool::ANY,
    ) {
        let candidate = if upper {
            lang.to_ascii_uppercase()
        } else {
            lang.to_string()
        };
        prop_assert!(supports_language(&candidate));
    }
}
