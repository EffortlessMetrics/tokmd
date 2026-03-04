//! BDD-style scenario tests simulating import graph patterns.
//!
//! Although this crate only parses and normalizes imports, these tests
//! verify that the extracted edges are correct for graph construction
//! by higher-tier crates.

use std::collections::{BTreeMap, BTreeSet};

use tokmd_analysis_imports::{normalize_import_target, parse_imports};

/// Helper: build an adjacency list from files and their source lines.
fn build_import_graph(files: &[(&str, &str, &[&str])]) -> BTreeMap<String, BTreeSet<String>> {
    let mut graph: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for &(file, lang, lines) in files {
        let raw = parse_imports(lang, lines);
        let targets: BTreeSet<String> = raw.iter().map(|t| normalize_import_target(t)).collect();
        graph.insert(file.to_string(), targets);
    }
    graph
}

// ── Scenario: Linear dependency chain ──────────────────────────────

#[test]
fn scenario_linear_chain_a_imports_b_imports_c() {
    // Given three Rust files forming a linear chain: a -> b -> c
    let files: Vec<(&str, &str, &[&str])> = vec![
        ("a.rs", "rust", &["use b::Thing;"]),
        ("b.rs", "rust", &["use c::Other;"]),
        ("c.rs", "rust", &[]),
    ];

    let graph = build_import_graph(&files);

    // Then a depends on b, b depends on c, c depends on nothing
    assert!(graph["a.rs"].contains("b"));
    assert!(graph["b.rs"].contains("c"));
    assert!(graph["c.rs"].is_empty());
}

// ── Scenario: Circular dependency ──────────────────────────────────

#[test]
fn scenario_circular_dependency_a_and_b_import_each_other() {
    // Given two Python files that import each other
    let files: Vec<(&str, &str, &[&str])> = vec![
        ("a.py", "python", &["import b"]),
        ("b.py", "python", &["import a"]),
    ];

    let graph = build_import_graph(&files);

    // Then both nodes have edges to each other
    assert!(graph["a.py"].contains("b"));
    assert!(graph["b.py"].contains("a"));
}

#[test]
fn scenario_three_way_circular_dependency() {
    // Given A -> B -> C -> A forming a cycle
    let files: Vec<(&str, &str, &[&str])> = vec![
        ("a.py", "python", &["import b"]),
        ("b.py", "python", &["import c"]),
        ("c.py", "python", &["import a"]),
    ];

    let graph = build_import_graph(&files);

    assert!(graph["a.py"].contains("b"));
    assert!(graph["b.py"].contains("c"));
    assert!(graph["c.py"].contains("a"));
}

// ── Scenario: Self-import ──────────────────────────────────────────

#[test]
fn scenario_self_import_via_relative_normalizes_to_local() {
    // Given a JS file that imports itself via relative path
    let files: Vec<(&str, &str, &[&str])> = vec![(
        "index.js",
        "javascript",
        &[r#"import self from "./index";"#],
    )];

    let graph = build_import_graph(&files);

    // Then the self-reference normalizes to "local"
    assert!(graph["index.js"].contains("local"));
}

#[test]
fn scenario_rust_self_import_uses_crate_keyword() {
    // Given a Rust file using `use crate::` (self-referencing the current crate)
    let files: Vec<(&str, &str, &[&str])> = vec![(
        "lib.rs",
        "rust",
        &["use crate::models::User;", "use crate::config::Settings;"],
    )];

    let graph = build_import_graph(&files);

    // Then the dependency root is "crate" (a self-reference marker)
    assert_eq!(graph["lib.rs"].len(), 1);
    assert!(graph["lib.rs"].contains("crate"));
}

// ── Scenario: Diamond dependency ───────────────────────────────────

#[test]
fn scenario_diamond_dependency_pattern() {
    // Given: A -> B, A -> C, B -> D, C -> D (diamond)
    let files: Vec<(&str, &str, &[&str])> = vec![
        ("a.py", "python", &["import b", "import c"]),
        ("b.py", "python", &["import d"]),
        ("c.py", "python", &["import d"]),
        ("d.py", "python", &[]),
    ];

    let graph = build_import_graph(&files);

    assert_eq!(graph["a.py"].len(), 2);
    assert!(graph["a.py"].contains("b"));
    assert!(graph["a.py"].contains("c"));
    assert!(graph["b.py"].contains("d"));
    assert!(graph["c.py"].contains("d"));
    assert!(graph["d.py"].is_empty());
}

// ── Scenario: Hub/spoke pattern ────────────────────────────────────

#[test]
fn scenario_hub_module_imports_many_spokes() {
    // Given a hub file that imports many dependencies
    let lines: Vec<String> = (0..20).map(|i| format!("use spoke_{i}::Api;")).collect();
    let refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();

    let files: Vec<(&str, &str, &[&str])> = vec![("hub.rs", "rust", &refs)];
    let graph = build_import_graph(&files);

    assert_eq!(graph["hub.rs"].len(), 20);
    for i in 0..20 {
        assert!(graph["hub.rs"].contains(&format!("spoke_{i}")));
    }
}

// ── Scenario: Mixed languages in a monorepo ────────────────────────

#[test]
fn scenario_polyglot_monorepo_graph() {
    let files: Vec<(&str, &str, &[&str])> = vec![
        (
            "api/main.go",
            "go",
            &[r#"import "fmt""#, r#"import "net/http""#],
        ),
        (
            "web/app.js",
            "javascript",
            &[r#"import React from "react";"#],
        ),
        ("ml/train.py", "python", &["import numpy", "import pandas"]),
        (
            "core/lib.rs",
            "rust",
            &["use serde::Serialize;", "mod config;"],
        ),
    ];

    let graph = build_import_graph(&files);

    assert_eq!(graph.len(), 4);
    assert!(graph["api/main.go"].contains("fmt"));
    assert!(graph["web/app.js"].contains("react"));
    assert!(graph["ml/train.py"].contains("numpy"));
    assert!(graph["core/lib.rs"].contains("serde"));
}

// ── Scenario: Deduplication of normalized targets ──────────────────

#[test]
fn scenario_multiple_imports_from_same_root_deduplicate_in_graph() {
    // Given a Rust file with multiple imports from the same crate
    let files: Vec<(&str, &str, &[&str])> = vec![(
        "lib.rs",
        "rust",
        &[
            "use serde::Serialize;",
            "use serde::Deserialize;",
            "use serde_json::Value;",
            "use serde_json::Map;",
        ],
    )];

    let graph = build_import_graph(&files);

    // BTreeSet deduplicates: only "serde" and "serde_json"
    assert_eq!(graph["lib.rs"].len(), 2);
    assert!(graph["lib.rs"].contains("serde"));
    assert!(graph["lib.rs"].contains("serde_json"));
}

// ── Scenario: All relative imports collapse to single "local" node ─

#[test]
fn scenario_all_relative_js_imports_collapse_to_local() {
    let files: Vec<(&str, &str, &[&str])> = vec![(
        "app.js",
        "javascript",
        &[
            r#"import utils from "./utils";"#,
            r#"import config from "../config";"#,
            r#"import db from "./db/client";"#,
        ],
    )];

    let graph = build_import_graph(&files);

    // All three relative imports normalize to "local"
    assert_eq!(graph["app.js"].len(), 1);
    assert!(graph["app.js"].contains("local"));
}

// ── Scenario: File with no imports (leaf node) ─────────────────────

#[test]
fn scenario_leaf_files_have_empty_dependency_sets() {
    let files: Vec<(&str, &str, &[&str])> = vec![
        ("leaf.rs", "rust", &["fn main() {}", "let x = 42;"]),
        ("leaf.py", "python", &["x = 1", "print(x)"]),
        ("leaf.go", "go", &["func main() {}", "fmt.Println()"]),
        (
            "leaf.js",
            "javascript",
            &["const x = 1;", "console.log(x);"],
        ),
    ];

    let graph = build_import_graph(&files);

    for (file, deps) in &graph {
        assert!(deps.is_empty(), "{file} should have no imports");
    }
}

// ── Scenario: Deeply nested import paths ───────────────────────────

#[test]
fn scenario_deeply_nested_go_imports_normalize_to_domain_root() {
    let files: Vec<(&str, &str, &[&str])> = vec![(
        "main.go",
        "go",
        &[
            "import (",
            r#""github.com/org/repo/internal/pkg/subpkg/v2""#,
            r#""gitlab.com/team/project/cmd/server""#,
            r#""fmt""#,
            ")",
        ],
    )];

    let graph = build_import_graph(&files);

    // All Go module paths normalize to their first dot-separated segment
    let deps = &graph["main.go"];
    assert!(deps.contains("github"));
    assert!(deps.contains("gitlab"));
    assert!(deps.contains("fmt"));
}

// ── Scenario: Graph node count equals file count ───────────────────

#[test]
fn scenario_graph_has_one_node_per_file() {
    let files: Vec<(&str, &str, &[&str])> = vec![
        ("a.rs", "rust", &["use std::io;"]),
        ("b.py", "python", &["import os"]),
        ("c.js", "javascript", &[r#"import x from "y";"#]),
        ("d.go", "go", &[r#"import "fmt""#]),
    ];

    let graph = build_import_graph(&files);
    assert_eq!(graph.len(), 4);
}

// ── Scenario: Isolated nodes (no imports, no importers) ────────────

#[test]
fn scenario_isolated_files_produce_graph_with_all_empty_sets() {
    let files: Vec<(&str, &str, &[&str])> = vec![
        ("orphan1.rs", "rust", &["// no imports here"]),
        ("orphan2.py", "python", &["x = 42"]),
    ];

    let graph = build_import_graph(&files);

    assert!(graph["orphan1.rs"].is_empty());
    assert!(graph["orphan2.py"].is_empty());
}

// ── Scenario: Star topology (many files importing one hub) ─────────

#[test]
fn scenario_star_topology_many_import_one() {
    let files: Vec<(&str, &str, &[&str])> = vec![
        ("a.py", "python", &["import hub"]),
        ("b.py", "python", &["import hub"]),
        ("c.py", "python", &["import hub"]),
        ("hub.py", "python", &[]),
    ];

    let graph = build_import_graph(&files);

    for name in &["a.py", "b.py", "c.py"] {
        assert!(graph[*name].contains("hub"));
    }
    assert!(graph["hub.py"].is_empty());
}
