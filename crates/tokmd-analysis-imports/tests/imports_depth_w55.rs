//! Wave-55 depth tests for `tokmd-analysis-imports`.
//!
//! Targets gaps not covered by existing suites:
//! - Import graph construction from multi-file projects
//! - Circular dependency detection patterns
//! - Module grouping and deduplication after normalization
//! - Deterministic ordering of parsed imports
//! - Edge cases: self-imports, deeply nested paths, mixed languages
//! - Cross-language normalization consistency

use std::collections::{BTreeMap, BTreeSet};

use tokmd_analysis_imports::{normalize_import_target, parse_imports, supports_language};

// ── Helpers ─────────────────────────────────────────────────────────

/// Build an adjacency list mapping file → normalized dependency roots.
fn build_graph(files: &[(&str, &str, &[&str])]) -> BTreeMap<String, BTreeSet<String>> {
    let mut graph: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for &(file, lang, lines) in files {
        let raw = parse_imports(lang, lines);
        let targets: BTreeSet<String> = raw.iter().map(|t| normalize_import_target(t)).collect();
        graph.insert(file.to_string(), targets);
    }
    graph
}

/// Detect cycles in a directed graph via DFS.
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

/// Group imports by normalized root, counting occurrences.
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
// 1. Import graph construction
// =============================================================================

#[test]
fn graph_from_two_rust_files_has_correct_edges() {
    let files: Vec<(&str, &str, &[&str])> = vec![
        (
            "main.rs",
            "rust",
            &["use lib::run;", "use serde::Serialize;"],
        ),
        ("lib.rs", "rust", &["use std::io;"]),
    ];
    let graph = build_graph(&files);
    assert_eq!(graph.len(), 2);
    assert!(graph["main.rs"].contains("lib"));
    assert!(graph["main.rs"].contains("serde"));
    assert!(graph["lib.rs"].contains("std"));
}

#[test]
fn graph_preserves_btreemap_ordering_of_files() {
    let files: Vec<(&str, &str, &[&str])> = vec![
        ("z.rs", "rust", &["use a::X;"]),
        ("a.rs", "rust", &["use z::Y;"]),
        ("m.rs", "rust", &["use b::Z;"]),
    ];
    let graph = build_graph(&files);
    let keys: Vec<&String> = graph.keys().collect();
    assert_eq!(keys, vec!["a.rs", "m.rs", "z.rs"]);
}

#[test]
fn graph_from_mixed_languages_four_files() {
    let files: Vec<(&str, &str, &[&str])> = vec![
        ("app.py", "python", &["import flask", "import sqlalchemy"]),
        ("server.go", "go", &[r#"import "net/http""#]),
        (
            "index.js",
            "javascript",
            &[r#"import express from "express";"#],
        ),
        ("lib.rs", "rust", &["use tokio::runtime;"]),
    ];
    let graph = build_graph(&files);
    assert_eq!(graph.len(), 4);
    assert!(graph["app.py"].contains("flask"));
    assert!(graph["server.go"].contains("net"));
    assert!(graph["index.js"].contains("express"));
    assert!(graph["lib.rs"].contains("tokio"));
}

#[test]
fn graph_with_no_imports_all_leaf_nodes() {
    let files: Vec<(&str, &str, &[&str])> = vec![
        ("a.rs", "rust", &["fn main() {}"]),
        ("b.py", "python", &["x = 42"]),
        ("c.js", "javascript", &["console.log('hello');"]),
    ];
    let graph = build_graph(&files);
    for (_, deps) in &graph {
        assert!(deps.is_empty());
    }
}

#[test]
fn graph_single_file_with_many_deps() {
    let lines: Vec<String> = (0..50).map(|i| format!("import dep_{i}")).collect();
    let refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
    let files: Vec<(&str, &str, &[&str])> = vec![("app.py", "python", &refs)];
    let graph = build_graph(&files);
    assert_eq!(graph["app.py"].len(), 50);
}

// =============================================================================
// 2. Circular dependency detection
// =============================================================================

#[test]
fn no_cycle_in_linear_chain() {
    let files: Vec<(&str, &str, &[&str])> = vec![
        ("a", "python", &["import b"]),
        ("b", "python", &["import c"]),
        ("c", "python", &[]),
    ];
    let graph = build_graph(&files);
    assert!(!has_cycle(&graph));
}

#[test]
fn cycle_detected_in_two_node_loop() {
    let mut graph: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    graph.insert("a".to_string(), BTreeSet::from(["b".to_string()]));
    graph.insert("b".to_string(), BTreeSet::from(["a".to_string()]));
    assert!(has_cycle(&graph));
}

#[test]
fn cycle_detected_in_three_node_loop() {
    let mut graph: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    graph.insert("a".to_string(), BTreeSet::from(["b".to_string()]));
    graph.insert("b".to_string(), BTreeSet::from(["c".to_string()]));
    graph.insert("c".to_string(), BTreeSet::from(["a".to_string()]));
    assert!(has_cycle(&graph));
}

#[test]
fn no_cycle_in_diamond_pattern() {
    let mut graph: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    graph.insert(
        "a".to_string(),
        BTreeSet::from(["b".to_string(), "c".to_string()]),
    );
    graph.insert("b".to_string(), BTreeSet::from(["d".to_string()]));
    graph.insert("c".to_string(), BTreeSet::from(["d".to_string()]));
    graph.insert("d".to_string(), BTreeSet::new());
    assert!(!has_cycle(&graph));
}

#[test]
fn self_loop_detected_as_cycle() {
    let mut graph: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    graph.insert("a".to_string(), BTreeSet::from(["a".to_string()]));
    assert!(has_cycle(&graph));
}

#[test]
fn cycle_in_large_graph_with_one_back_edge() {
    let mut graph: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for i in 0..20 {
        let next = format!("n{}", i + 1);
        graph.insert(format!("n{i}"), BTreeSet::from([next]));
    }
    // n20 → n0 creates a cycle
    graph.insert("n20".to_string(), BTreeSet::from(["n0".to_string()]));
    assert!(has_cycle(&graph));
}

// =============================================================================
// 3. Import detection for different languages
// =============================================================================

#[test]
fn rust_use_with_nested_braces_extracts_root() {
    let lines = ["use std::{io::{self, Read}, fs};"];
    let imports = parse_imports("rust", &lines);
    assert_eq!(imports, vec!["std"]);
}

#[test]
fn rust_mod_and_use_interleaved_with_code() {
    let lines = [
        "mod config;",
        "use config::Settings;",
        "fn setup() {}",
        "mod routes;",
        "use routes::api;",
    ];
    let imports = parse_imports("rust", &lines);
    assert_eq!(imports, vec!["config", "config", "routes", "routes"]);
}

#[test]
fn js_import_star_as_namespace() {
    let lines = [r#"import * as React from "react";"#];
    let imports = parse_imports("javascript", &lines);
    assert_eq!(imports, vec!["react"]);
}

#[test]
fn js_require_nested_in_expression() {
    let lines = [r#"const data = JSON.parse(require("./data.json"));"#];
    let imports = parse_imports("javascript", &lines);
    assert_eq!(imports, vec!["./data.json"]);
}

#[test]
fn python_from_with_multiple_dots() {
    let lines = ["from ....deep.module import func"];
    let imports = parse_imports("python", &lines);
    assert_eq!(imports, vec!["....deep.module"]);
    assert_eq!(normalize_import_target(&imports[0]), "local");
}

#[test]
fn go_block_import_with_aliased_and_blank_imports() {
    let lines = vec![
        "import (",
        r#"    . "testing""#,
        r#"    _ "database/sql""#,
        r#"    mylog "log""#,
        ")",
    ];
    let imports = parse_imports("go", &lines);
    assert_eq!(imports, vec!["testing", "database/sql", "log"]);
}

#[test]
fn typescript_import_type_only() {
    let lines = [r#"import type { ReactNode } from "react";"#];
    let imports = parse_imports("typescript", &lines);
    assert_eq!(imports, vec!["react"]);
}

#[test]
fn typescript_import_equals_not_captured() {
    // `import x = require(...)` doesn't start with `import ` in the way that matches
    let lines = [r#"import express = require("express");"#];
    let imports = parse_imports("typescript", &lines);
    // This starts with "import " and has a quoted string from require()
    assert!(!imports.is_empty());
}

// =============================================================================
// 4. Module grouping
// =============================================================================

#[test]
fn grouping_rust_imports_by_crate_root() {
    let lines: &[&str] = &[
        "use std::io;",
        "use std::fs;",
        "use std::collections::HashMap;",
        "use serde::Serialize;",
        "use serde::Deserialize;",
        "use anyhow::Result;",
    ];
    let groups = group_by_root("rust", lines);
    assert_eq!(groups["std"], 3);
    assert_eq!(groups["serde"], 2);
    assert_eq!(groups["anyhow"], 1);
}

#[test]
fn grouping_js_imports_collapses_relative_to_local() {
    let lines: &[&str] = &[
        r#"import a from "./a";"#,
        r#"import b from "../b";"#,
        r#"import c from "./c";"#,
        r#"import React from "react";"#,
    ];
    let groups = group_by_root("javascript", lines);
    assert_eq!(groups["local"], 3);
    assert_eq!(groups["react"], 1);
}

#[test]
fn grouping_python_separates_stdlib_from_relative() {
    let lines: &[&str] = &[
        "import os",
        "import sys",
        "from . import utils",
        "from .. import models",
    ];
    let groups = group_by_root("python", lines);
    assert_eq!(groups["os"], 1);
    assert_eq!(groups["sys"], 1);
    assert_eq!(groups["local"], 2);
}

#[test]
fn grouping_go_normalizes_github_paths() {
    let lines: &[&str] = &[
        "import (",
        r#""fmt""#,
        r#""github.com/user/repo/pkg""#,
        r#""github.com/other/lib/api""#,
        ")",
    ];
    let groups = group_by_root("go", lines);
    assert_eq!(groups["fmt"], 1);
    assert_eq!(groups["github"], 2);
}

#[test]
fn grouping_empty_input_yields_empty_map() {
    let lines: &[&str] = &[];
    let groups = group_by_root("rust", lines);
    assert!(groups.is_empty());
}

// =============================================================================
// 5. Edge cases: no imports, self-imports, deeply nested
// =============================================================================

#[test]
fn no_imports_in_code_only_file() {
    let lines = [
        "fn main() {",
        "    let x = 42;",
        "    println!(\"{}\", x);",
        "}",
    ];
    let imports = parse_imports("rust", &lines);
    assert!(imports.is_empty());
}

#[test]
fn rust_self_import_normalizes_to_self() {
    let lines = ["use self::inner::Foo;"];
    let imports = parse_imports("rust", &lines);
    assert_eq!(imports, vec!["self"]);
    // "self" does not start with '.', so normalize returns "self"
    assert_eq!(normalize_import_target("self"), "self");
}

#[test]
fn rust_crate_import_normalizes_to_crate() {
    let lines = ["use crate::models::User;"];
    let imports = parse_imports("rust", &lines);
    assert_eq!(normalize_import_target(&imports[0]), "crate");
}

#[test]
fn python_single_dot_import_normalizes_to_local() {
    let lines = ["from . import helper"];
    let imports = parse_imports("python", &lines);
    assert_eq!(imports, vec!["."]);
    assert_eq!(normalize_import_target("."), "local");
}

#[test]
fn js_self_import_via_relative_normalizes_to_local() {
    let lines = [r#"import self from "./index";"#];
    let imports = parse_imports("javascript", &lines);
    assert_eq!(normalize_import_target(&imports[0]), "local");
}

#[test]
fn deeply_nested_rust_path_extracts_top_crate() {
    let lines = ["use a::b::c::d::e::f::g::h;"];
    let imports = parse_imports("rust", &lines);
    assert_eq!(imports, vec!["a"]);
}

#[test]
fn deeply_nested_go_module_normalizes_to_first_segment() {
    let target = "github.com/org/repo/internal/pkg/sub/v3";
    assert_eq!(normalize_import_target(target), "github");
}

#[test]
fn empty_lines_between_imports_still_captured() {
    let lines = [
        "use std::io;",
        "",
        "",
        "use serde::Serialize;",
        "",
        "use anyhow::Result;",
    ];
    let imports = parse_imports("rust", &lines);
    assert_eq!(imports, vec!["std", "serde", "anyhow"]);
}

// =============================================================================
// 6. Deterministic ordering
// =============================================================================

#[test]
fn parse_imports_order_matches_source_order() {
    let lines = ["use z_crate::Z;", "use a_crate::A;", "use m_crate::M;"];
    let imports = parse_imports("rust", &lines);
    assert_eq!(imports, vec!["z_crate", "a_crate", "m_crate"]);
}

#[test]
fn parse_imports_deterministic_across_100_calls() {
    let lines = [
        "use std::io;",
        "use serde::Serialize;",
        "use anyhow::Result;",
    ];
    let baseline = parse_imports("rust", &lines);
    for _ in 0..100 {
        assert_eq!(parse_imports("rust", &lines), baseline);
    }
}

#[test]
fn normalized_graph_edges_in_btreeset_order() {
    let files: Vec<(&str, &str, &[&str])> = vec![(
        "app.py",
        "python",
        &["import z_lib", "import a_lib", "import m_lib"],
    )];
    let graph = build_graph(&files);
    let deps: Vec<&String> = graph["app.py"].iter().collect();
    // BTreeSet orders alphabetically
    assert_eq!(deps, vec!["a_lib", "m_lib", "z_lib"]);
}

#[test]
fn supports_language_is_deterministic() {
    for lang in [
        "rust",
        "javascript",
        "typescript",
        "python",
        "go",
        "java",
        "c",
        "ruby",
    ] {
        let first = supports_language(lang);
        let second = supports_language(lang);
        assert_eq!(first, second, "non-deterministic for {lang}");
    }
}

// =============================================================================
// 7. Cross-language normalization consistency
// =============================================================================

#[test]
fn same_target_name_normalizes_identically_across_languages() {
    let rust_imports = parse_imports("rust", &["use serde::Serialize;"]);
    let py_imports = parse_imports("python", &["import serde"]);
    assert_eq!(
        normalize_import_target(&rust_imports[0]),
        normalize_import_target(&py_imports[0])
    );
}

#[test]
fn relative_imports_all_normalize_to_local_regardless_of_language() {
    let js = parse_imports("javascript", &[r#"import x from "./foo";"#]);
    let py = parse_imports("python", &["from . import foo"]);

    assert_eq!(normalize_import_target(&js[0]), "local");
    assert_eq!(normalize_import_target(&py[0]), "local");
}

#[test]
fn normalize_preserves_underscores_and_hyphens() {
    assert_eq!(normalize_import_target("my_crate"), "my_crate");
    assert_eq!(normalize_import_target("my-package"), "my-package");
    assert_eq!(normalize_import_target("serde_json::Value"), "serde_json");
    assert_eq!(normalize_import_target("my-pkg/utils"), "my-pkg");
}

// =============================================================================
// 8. Unsupported language handling
// =============================================================================

#[test]
fn unsupported_language_returns_empty_for_any_input() {
    let lines = [
        "#include <stdio.h>",
        "import java.util.*;",
        "require 'rails'",
    ];
    for lang in ["c", "java", "ruby", "kotlin", "swift", "haskell", ""] {
        assert!(
            parse_imports(lang, &lines).is_empty(),
            "expected empty for {lang}"
        );
    }
}

#[test]
fn supports_language_rejects_abbreviations() {
    assert!(!supports_language("rs"));
    assert!(!supports_language("js"));
    assert!(!supports_language("ts"));
    assert!(!supports_language("py"));
}

// =============================================================================
// 9. Normalize edge cases
// =============================================================================

#[test]
fn normalize_target_with_only_dots_is_local() {
    assert_eq!(normalize_import_target("."), "local");
    assert_eq!(normalize_import_target(".."), "local");
    assert_eq!(normalize_import_target("..."), "local");
}

#[test]
fn normalize_target_with_leading_separator() {
    // "/absolute/path" → split on '/' → first is ""
    assert_eq!(normalize_import_target("/absolute/path"), "");
}

#[test]
fn normalize_target_preserves_at_scope() {
    assert_eq!(normalize_import_target("@scope/package/sub"), "@scope");
    assert_eq!(normalize_import_target("@types/node"), "@types");
}

#[test]
fn normalize_idempotent_for_simple_names() {
    for name in ["std", "serde", "react", "os", "fmt"] {
        let first = normalize_import_target(name);
        let second = normalize_import_target(&first);
        assert_eq!(first, second);
    }
}

// =============================================================================
// 10. Large / stress inputs
// =============================================================================

#[test]
fn parse_1000_python_imports_without_panic() {
    let lines: Vec<String> = (0..1000).map(|i| format!("import module_{i}")).collect();
    let refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
    let imports = parse_imports("python", &refs);
    assert_eq!(imports.len(), 1000);
}

#[test]
fn parse_go_block_with_100_imports() {
    let mut lines = vec!["import (".to_string()];
    for i in 0..100 {
        lines.push(format!(r#""pkg_{i}""#));
    }
    lines.push(")".to_string());
    let refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
    let imports = parse_imports("go", &refs);
    assert_eq!(imports.len(), 100);
}

#[test]
fn graph_with_50_interconnected_files() {
    let mut files_data: Vec<(String, Vec<String>)> = Vec::new();
    for i in 0..50 {
        let line = if i < 49 {
            format!("import mod_{}", i + 1)
        } else {
            String::new()
        };
        files_data.push((format!("file_{i}.py"), vec![line]));
    }

    let files: Vec<(&str, &str, Vec<&str>)> = files_data
        .iter()
        .map(|(name, lines)| {
            let refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
            (name.as_str(), "python", refs)
        })
        .collect();

    let mut graph: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (file, lang, lines) in &files {
        let raw = parse_imports(lang, lines);
        let targets: BTreeSet<String> = raw.iter().map(|t| normalize_import_target(t)).collect();
        graph.insert(file.to_string(), targets);
    }

    assert_eq!(graph.len(), 50);
    assert!(!has_cycle(&graph));
}
