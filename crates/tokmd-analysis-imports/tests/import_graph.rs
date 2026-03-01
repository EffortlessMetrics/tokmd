//! Tests for import extraction across languages, graph construction
//! simulation, cycle detection, and deterministic ordering.

use std::collections::{BTreeMap, BTreeSet};
use tokmd_analysis_imports::{normalize_import_target, parse_imports, supports_language};

// ── Graph construction helpers ──────────────────────────────────────

type ImportGraph = BTreeMap<String, BTreeSet<String>>;

/// Build a dependency graph: module name → set of normalized import roots.
fn build_import_graph(modules: &[(&str, &str, Vec<&str>)]) -> ImportGraph {
    let mut graph = ImportGraph::new();
    for (module_name, lang, lines) in modules {
        let imports = parse_imports(lang, lines);
        let targets: BTreeSet<String> =
            imports.iter().map(|t| normalize_import_target(t)).collect();
        graph.insert(module_name.to_string(), targets);
    }
    graph
}

/// DFS-based cycle detection on the import graph.
fn has_cycle(graph: &ImportGraph) -> bool {
    let mut visited = BTreeSet::new();
    let mut in_stack = BTreeSet::new();
    for node in graph.keys() {
        if dfs_cycle(node, graph, &mut visited, &mut in_stack) {
            return true;
        }
    }
    false
}

fn dfs_cycle(
    node: &str,
    graph: &ImportGraph,
    visited: &mut BTreeSet<String>,
    in_stack: &mut BTreeSet<String>,
) -> bool {
    if in_stack.contains(node) {
        return true;
    }
    if visited.contains(node) {
        return false;
    }
    visited.insert(node.to_string());
    in_stack.insert(node.to_string());
    if let Some(neighbors) = graph.get(node) {
        for neighbor in neighbors {
            if dfs_cycle(neighbor, graph, visited, in_stack) {
                return true;
            }
        }
    }
    in_stack.remove(node);
    false
}

// ── Language support ────────────────────────────────────────────────

#[test]
fn java_is_not_supported() {
    assert!(!supports_language("java"));
    assert!(!supports_language("Java"));
    assert!(!supports_language("JAVA"));
}

#[test]
fn java_parsing_returns_empty() {
    let lines = vec!["import java.util.List;", "import com.example.MyClass;"];
    assert!(parse_imports("java", &lines).is_empty());
}

#[test]
fn all_five_supported_languages() {
    for lang in &["rust", "javascript", "typescript", "python", "go"] {
        assert!(supports_language(lang), "{lang} should be supported");
    }
}

// ── Multi-language import extraction ────────────────────────────────

#[test]
fn rust_project_imports() {
    let lines = vec![
        "use serde::Serialize;",
        "use std::collections::BTreeMap;",
        "use anyhow::Result;",
        "mod internal;",
    ];
    let imports = parse_imports("rust", &lines);
    assert_eq!(imports, vec!["serde", "std", "anyhow", "internal"]);
}

#[test]
fn python_project_imports() {
    let lines = vec![
        "import os",
        "from pathlib import Path",
        "import json",
        "from . import utils",
    ];
    let imports = parse_imports("python", &lines);
    assert_eq!(imports, vec!["os", "pathlib", "json", "."]);
}

#[test]
fn javascript_project_imports() {
    let lines = vec![
        r#"import React from "react";"#,
        r#"import { useState } from "react";"#,
        r#"const fs = require("fs");"#,
        r#"import "./styles.css";"#,
    ];
    let imports = parse_imports("javascript", &lines);
    assert_eq!(imports, vec!["react", "react", "fs", "./styles.css"]);
}

#[test]
fn go_project_imports() {
    let lines = vec![
        "import (",
        r#""fmt""#,
        r#""net/http""#,
        r#""github.com/gorilla/mux""#,
        ")",
    ];
    let imports = parse_imports("go", &lines);
    assert_eq!(imports, vec!["fmt", "net/http", "github.com/gorilla/mux"]);
}

// ── Graph construction ──────────────────────────────────────────────

#[test]
fn build_graph_from_rust_modules() {
    let modules: Vec<(&str, &str, Vec<&str>)> = vec![
        (
            "app",
            "rust",
            vec!["use config::Settings;", "use db::Pool;"],
        ),
        ("config", "rust", vec!["use serde::Deserialize;"]),
        ("db", "rust", vec!["use config::Settings;"]),
    ];
    let graph = build_import_graph(&modules);

    assert_eq!(graph.len(), 3);
    assert!(graph["app"].contains("config"));
    assert!(graph["app"].contains("db"));
    assert!(graph["config"].contains("serde"));
    assert!(graph["db"].contains("config"));
}

#[test]
fn build_graph_from_python_modules() {
    let modules: Vec<(&str, &str, Vec<&str>)> = vec![
        (
            "app",
            "python",
            vec!["import models", "from views import render"],
        ),
        ("models", "python", vec!["import db"]),
        ("views", "python", vec!["import models"]),
    ];
    let graph = build_import_graph(&modules);

    assert_eq!(graph.len(), 3);
    assert!(graph["app"].contains("models"));
    assert!(graph["app"].contains("views"));
    assert!(graph["models"].contains("db"));
    assert!(graph["views"].contains("models"));
}

#[test]
fn build_graph_from_mixed_languages() {
    let modules: Vec<(&str, &str, Vec<&str>)> = vec![
        (
            "backend",
            "rust",
            vec!["use serde::Serialize;", "use tokio::Runtime;"],
        ),
        (
            "frontend",
            "javascript",
            vec![
                r#"import React from "react";"#,
                r#"const axios = require("axios");"#,
            ],
        ),
        (
            "scripts",
            "python",
            vec!["import os", "from pathlib import Path"],
        ),
        ("server", "go", vec![r#"import "net/http""#]),
    ];
    let graph = build_import_graph(&modules);

    assert_eq!(graph.len(), 4);
    assert!(graph["backend"].contains("serde"));
    assert!(graph["backend"].contains("tokio"));
    assert!(graph["frontend"].contains("react"));
    assert!(graph["frontend"].contains("axios"));
    assert!(graph["scripts"].contains("os"));
    assert!(graph["scripts"].contains("pathlib"));
    assert!(graph["server"].contains("net"));
}

#[test]
fn empty_modules_produce_empty_graph() {
    let modules: Vec<(&str, &str, Vec<&str>)> = vec![];
    let graph = build_import_graph(&modules);
    assert!(graph.is_empty());
}

#[test]
fn modules_with_no_imports_have_empty_edge_sets() {
    let modules: Vec<(&str, &str, Vec<&str>)> = vec![
        ("main", "rust", vec!["fn main() {}", "let x = 5;"]),
        ("utils", "python", vec!["# no imports here", "x = 42"]),
    ];
    let graph = build_import_graph(&modules);

    assert_eq!(graph.len(), 2);
    assert!(graph["main"].is_empty());
    assert!(graph["utils"].is_empty());
}

// ── Cycle detection ─────────────────────────────────────────────────

#[test]
fn detect_direct_cycle_in_rust_modules() {
    // foo imports bar, bar imports foo → cycle
    let modules: Vec<(&str, &str, Vec<&str>)> = vec![
        ("foo", "rust", vec!["use bar::Thing;"]),
        ("bar", "rust", vec!["use foo::Other;"]),
    ];
    let graph = build_import_graph(&modules);
    assert!(has_cycle(&graph), "direct mutual dependency is a cycle");
}

#[test]
fn detect_transitive_cycle_in_python() {
    // a→b, b→c, c→a
    let modules: Vec<(&str, &str, Vec<&str>)> = vec![
        ("a", "python", vec!["import b"]),
        ("b", "python", vec!["import c"]),
        ("c", "python", vec!["import a"]),
    ];
    let graph = build_import_graph(&modules);
    assert!(has_cycle(&graph), "transitive cycle a→b→c→a");
}

#[test]
fn no_cycle_in_acyclic_graph() {
    let modules: Vec<(&str, &str, Vec<&str>)> = vec![
        (
            "app",
            "rust",
            vec!["use config::Settings;", "use db::Pool;"],
        ),
        ("config", "rust", vec!["use serde::Deserialize;"]),
        ("db", "rust", vec!["use serde::Deserialize;"]),
    ];
    let graph = build_import_graph(&modules);
    assert!(!has_cycle(&graph), "DAG should have no cycle");
}

#[test]
fn no_cycle_in_star_topology() {
    // hub imports a, b, c — no back-edges
    let modules: Vec<(&str, &str, Vec<&str>)> = vec![
        ("hub", "python", vec!["import a", "import b", "import c"]),
        ("a", "python", vec!["import os"]),
        ("b", "python", vec!["import json"]),
        ("c", "python", vec!["import sys"]),
    ];
    let graph = build_import_graph(&modules);
    assert!(!has_cycle(&graph));
}

#[test]
fn self_import_detected_as_cycle() {
    // Module imports itself
    let modules: Vec<(&str, &str, Vec<&str>)> =
        vec![("self_ref", "python", vec!["import self_ref"])];
    let graph = build_import_graph(&modules);
    assert!(has_cycle(&graph), "self-import is a cycle");
}

// ── Deterministic ordering ──────────────────────────────────────────

#[test]
fn graph_keys_are_sorted() {
    let modules: Vec<(&str, &str, Vec<&str>)> = vec![
        ("zebra", "rust", vec!["use alpha::A;"]),
        ("alpha", "rust", vec!["use zebra::Z;"]),
        ("middle", "rust", vec!["use alpha::A;"]),
    ];
    let graph = build_import_graph(&modules);
    let keys: Vec<&String> = graph.keys().collect();
    assert_eq!(
        keys.iter().map(|k| k.as_str()).collect::<Vec<_>>(),
        vec!["alpha", "middle", "zebra"],
        "BTreeMap keys must be sorted"
    );
}

#[test]
fn graph_edge_sets_are_sorted() {
    let modules: Vec<(&str, &str, Vec<&str>)> = vec![(
        "app",
        "rust",
        vec!["use zebra::Z;", "use alpha::A;", "use middle::M;"],
    )];
    let graph = build_import_graph(&modules);
    let edges: Vec<&String> = graph["app"].iter().collect();
    assert_eq!(
        edges.iter().map(|e| e.as_str()).collect::<Vec<_>>(),
        vec!["alpha", "middle", "zebra"],
        "BTreeSet edges must be sorted"
    );
}

#[test]
fn graph_construction_is_deterministic() {
    let modules: Vec<(&str, &str, Vec<&str>)> = vec![
        (
            "main",
            "rust",
            vec![
                "use serde::Serialize;",
                "use tokio::Runtime;",
                "mod config;",
            ],
        ),
        (
            "config",
            "python",
            vec!["import os", "from pathlib import Path"],
        ),
        ("web", "javascript", vec![r#"import React from "react";"#]),
    ];

    let graph_a = build_import_graph(&modules);
    let graph_b = build_import_graph(&modules);
    assert_eq!(graph_a, graph_b, "graph construction must be deterministic");
}

// ── Normalization in graph context ──────────────────────────────────

#[test]
fn duplicate_imports_deduplicate_in_edge_set() {
    let lines = vec![
        r#"import { useState } from "react";"#,
        r#"import { useEffect } from "react";"#,
        r#"import React from "react";"#,
    ];
    let imports = parse_imports("javascript", &lines);
    assert_eq!(imports, vec!["react", "react", "react"]);

    // When collected into BTreeSet, duplicates collapse
    let unique: BTreeSet<String> = imports.iter().map(|t| normalize_import_target(t)).collect();
    assert_eq!(unique.len(), 1);
    assert!(unique.contains("react"));
}

#[test]
fn relative_imports_all_normalize_to_local() {
    let lines = vec![
        r#"import { foo } from "./foo";"#,
        r#"import { bar } from "../bar";"#,
        r#"import "./styles.css";"#,
    ];
    let imports = parse_imports("javascript", &lines);
    let normalized: BTreeSet<String> = imports.iter().map(|t| normalize_import_target(t)).collect();
    assert_eq!(normalized.len(), 1);
    assert!(normalized.contains("local"));
}

#[test]
fn go_module_paths_normalize_to_domain_root() {
    let lines = vec![
        "import (",
        r#""github.com/gorilla/mux""#,
        r#""github.com/sirupsen/logrus""#,
        r#""golang.org/x/sync""#,
        ")",
    ];
    let imports = parse_imports("go", &lines);
    let normalized: BTreeSet<String> = imports.iter().map(|t| normalize_import_target(t)).collect();

    // All github.com imports collapse to "github", golang.org to "golang"
    assert!(normalized.contains("github"));
    assert!(normalized.contains("golang"));
    assert_eq!(normalized.len(), 2);
}

// ── Edge count and node count ───────────────────────────────────────

#[test]
fn graph_node_count_matches_module_count() {
    let modules: Vec<(&str, &str, Vec<&str>)> = vec![
        ("a", "rust", vec!["use b::X;"]),
        ("b", "rust", vec!["use c::Y;"]),
        ("c", "rust", vec![]),
    ];
    let graph = build_import_graph(&modules);
    assert_eq!(graph.len(), 3);
}

#[test]
fn total_edge_count_in_simple_graph() {
    let modules: Vec<(&str, &str, Vec<&str>)> = vec![
        ("a", "rust", vec!["use b::X;", "use c::Y;"]),
        ("b", "rust", vec!["use c::Z;"]),
        ("c", "rust", vec![]),
    ];
    let graph = build_import_graph(&modules);
    let total_edges: usize = graph.values().map(|s| s.len()).sum();
    assert_eq!(total_edges, 3, "a→b, a→c, b→c");
}

// ── Large graph determinism ─────────────────────────────────────────

#[test]
fn large_module_set_is_deterministic() {
    let modules: Vec<(&str, &str, Vec<&str>)> = (0..50)
        .map(|i| {
            let name = Box::leak(format!("mod_{i}").into_boxed_str()) as &str;
            let dep = Box::leak(format!("use mod_{}::X;", (i + 1) % 50).into_boxed_str()) as &str;
            (name, "rust", vec![dep])
        })
        .collect();

    let graph_a = build_import_graph(&modules);
    let graph_b = build_import_graph(&modules);
    assert_eq!(graph_a, graph_b);
    assert_eq!(graph_a.len(), 50);
}
