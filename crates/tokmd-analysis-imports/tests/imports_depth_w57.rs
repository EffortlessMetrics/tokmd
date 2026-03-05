//! Wave-57 depth tests for `tokmd-analysis-imports`.
//!
//! Covers:
//! - Import extraction for Rust (use/mod), Python (import/from), JS/TS (import/require)
//! - Graph construction from import data
//! - Cycle detection
//! - Complex import patterns (re-exports, wildcard, glob)
//! - Deterministic ordering
//! - Normalization edge cases

use std::collections::{BTreeMap, BTreeSet};

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

// =============================================================================
// 1. Rust import extraction
// =============================================================================

#[test]
fn rust_use_with_nested_braces() {
    let lines = ["use std::collections::{HashMap, BTreeMap};"];
    let imports = parse_imports("Rust", &lines);
    assert_eq!(imports, vec!["std"]);
}

#[test]
fn rust_use_crate_prefix() {
    let lines = ["use crate::config::Settings;"];
    let imports = parse_imports("Rust", &lines);
    assert_eq!(imports, vec!["crate"]);
}

#[test]
fn rust_use_self_prefix() {
    let lines = ["use self::inner::Thing;"];
    let imports = parse_imports("Rust", &lines);
    assert_eq!(imports, vec!["self"]);
}

#[test]
fn rust_use_super_prefix() {
    let lines = ["use super::parent_mod;"];
    let imports = parse_imports("Rust", &lines);
    assert_eq!(imports, vec!["super"]);
}

#[test]
fn rust_mod_declaration_no_semicolon_trailing_space() {
    // mod with trailing whitespace before semicolon
    let lines = ["mod  utils ;"];
    let imports = parse_imports("Rust", &lines);
    assert_eq!(imports, vec!["utils"]);
}

#[test]
fn rust_pub_use_not_parsed_as_use() {
    // pub use is not "use " at start
    let lines = ["pub use crate::api;"];
    let imports = parse_imports("Rust", &lines);
    // starts with "pub", not "use", so no match
    assert!(imports.is_empty());
}

#[test]
fn rust_mixed_use_and_mod_ordering() {
    let lines = [
        "use z_crate::Z;",
        "mod b_mod;",
        "use a_crate::A;",
        "mod d_mod;",
    ];
    let imports = parse_imports("Rust", &lines);
    // Must preserve source order
    assert_eq!(imports, vec!["z_crate", "b_mod", "a_crate", "d_mod"]);
}

// =============================================================================
// 2. Python import extraction
// =============================================================================

#[test]
fn python_from_with_multiple_names() {
    let lines = ["from os.path import join, exists, basename"];
    let imports = parse_imports("Python", &lines);
    assert_eq!(imports, vec!["os.path"]);
}

#[test]
fn python_import_as_alias() {
    let lines = ["import numpy as np"];
    let imports = parse_imports("Python", &lines);
    // First whitespace-delimited token after "import " is "numpy"
    assert_eq!(imports, vec!["numpy"]);
}

#[test]
fn python_from_relative_import() {
    let lines = ["from . import utils", "from ..core import run"];
    let imports = parse_imports("Python", &lines);
    // Parser takes first whitespace token: "." and "..core"
    assert_eq!(imports, vec![".", "..core"]);
}

#[test]
fn python_normalization_of_relative_imports() {
    let lines = ["from . import utils", "from ..core import run"];
    let imports = parse_imports("Python", &lines);
    let normalized: Vec<String> = imports.iter().map(|t| normalize_import_target(t)).collect();
    // "." starts with '.', "..core" starts with '.' → both normalize to "local"
    assert_eq!(normalized, vec!["local", "local"]);
}

// =============================================================================
// 3. JavaScript / TypeScript import extraction
// =============================================================================

#[test]
fn js_dynamic_import_not_extracted() {
    // Dynamic import() is not a "require(" at beginning
    let lines = ["const mod = import('dynamic-mod');"];
    let imports = parse_imports("JavaScript", &lines);
    // "import(" doesn't start with "import " and this is `import(` not `require(`
    // Actually, the line starts with "const" so "import " check fails,
    // and there's no "require(" found. Let's verify.
    assert!(imports.is_empty());
}

#[test]
fn js_require_in_middle_of_line() {
    let lines = ["const x = require('middle-pkg');"];
    let imports = parse_imports("JavaScript", &lines);
    assert_eq!(imports, vec!["middle-pkg"]);
}

#[test]
fn js_import_with_default_and_named() {
    let lines = ["import React, { useState } from 'react';"];
    let imports = parse_imports("JavaScript", &lines);
    assert_eq!(imports, vec!["react"]);
}

#[test]
fn js_import_star_as() {
    let lines = ["import * as path from 'path';"];
    let imports = parse_imports("JavaScript", &lines);
    assert_eq!(imports, vec!["path"]);
}

#[test]
fn ts_import_type_only() {
    let lines = ["import type { Config } from 'config-pkg';"];
    let imports = parse_imports("TypeScript", &lines);
    assert_eq!(imports, vec!["config-pkg"]);
}

#[test]
fn js_relative_import_normalizes_to_local() {
    let lines = [
        "import foo from './foo';",
        "import bar from '../bar';",
        "const baz = require('./baz');",
    ];
    let imports = parse_imports("JavaScript", &lines);
    let normalized: Vec<String> = imports.iter().map(|t| normalize_import_target(t)).collect();
    assert_eq!(normalized, vec!["local", "local", "local"]);
}

// =============================================================================
// 4. Go import extraction
// =============================================================================

#[test]
fn go_aliased_import_in_block() {
    let lines = ["import (", "\tf \"fmt\"", "\t\"os\"", ")"];
    let imports = parse_imports("Go", &lines);
    // extract_quoted picks up "fmt" and "os"
    assert_eq!(imports, vec!["fmt", "os"]);
}

#[test]
fn go_blank_import() {
    let lines = ["import (", "\t_ \"database/sql\"", ")"];
    let imports = parse_imports("Go", &lines);
    assert_eq!(imports, vec!["database/sql"]);
}

// =============================================================================
// 5. Graph construction
// =============================================================================

#[test]
fn graph_from_mixed_languages() {
    let files: Vec<(&str, &str, &[&str])> = vec![
        ("main.rs", "rust", &["use serde::Serialize;"]),
        ("app.py", "python", &["import flask"]),
        ("index.js", "javascript", &["import React from 'react';"]),
    ];
    let graph = build_graph(&files);
    assert_eq!(graph.len(), 3);
    assert!(graph["main.rs"].contains("serde"));
    assert!(graph["app.py"].contains("flask"));
    assert!(graph["index.js"].contains("react"));
}

#[test]
fn graph_deduplicates_normalized_targets() {
    let files: Vec<(&str, &str, &[&str])> = vec![(
        "main.rs",
        "rust",
        &[
            "use std::io;",
            "use std::collections::HashMap;",
            "use std::fmt;",
        ],
    )];
    let graph = build_graph(&files);
    // BTreeSet deduplicates; all three normalize to "std"
    assert_eq!(graph["main.rs"].len(), 1);
    assert!(graph["main.rs"].contains("std"));
}

#[test]
fn graph_is_btreemap_sorted() {
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
// 6. Cycle detection
// =============================================================================

#[test]
fn no_cycle_in_dag() {
    let files: Vec<(&str, &str, &[&str])> = vec![
        ("a.rs", "rust", &["use b::X;"]),
        ("b.rs", "rust", &["use c::Y;"]),
        ("c.rs", "rust", &[]),
    ];
    let graph = build_graph(&files);
    assert!(!has_cycle(&graph));
}

#[test]
fn cycle_detected_in_mutual_dependency() {
    let mut graph: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    graph.insert("a".into(), BTreeSet::from(["b".into()]));
    graph.insert("b".into(), BTreeSet::from(["a".into()]));
    assert!(has_cycle(&graph));
}

#[test]
fn cycle_detected_in_three_node_cycle() {
    let mut graph: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    graph.insert("a".into(), BTreeSet::from(["b".into()]));
    graph.insert("b".into(), BTreeSet::from(["c".into()]));
    graph.insert("c".into(), BTreeSet::from(["a".into()]));
    assert!(has_cycle(&graph));
}

#[test]
fn no_cycle_when_all_nodes_isolated() {
    let mut graph: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    graph.insert("a".into(), BTreeSet::new());
    graph.insert("b".into(), BTreeSet::new());
    graph.insert("c".into(), BTreeSet::new());
    assert!(!has_cycle(&graph));
}

// =============================================================================
// 7. Deterministic ordering
// =============================================================================

#[test]
fn parse_preserves_source_order() {
    let lines = [
        "import z_mod",
        "import a_mod",
        "import m_mod",
        "from b_pkg import thing",
    ];
    let imports = parse_imports("Python", &lines);
    assert_eq!(imports, vec!["z_mod", "a_mod", "m_mod", "b_pkg"]);
}

#[test]
fn normalization_is_deterministic_across_calls() {
    let target = "react/dom/client";
    let r1 = normalize_import_target(target);
    let r2 = normalize_import_target(target);
    assert_eq!(r1, r2);
    assert_eq!(r1, "react");
}

// =============================================================================
// 8. supports_language coverage
// =============================================================================

#[test]
fn supports_language_case_insensitive() {
    assert!(supports_language("RUST"));
    assert!(supports_language("javascript"));
    assert!(supports_language("PYTHON"));
    assert!(supports_language("typescript"));
    assert!(supports_language("Go"));
}

#[test]
fn unsupported_language_returns_empty_imports() {
    let lines = ["#include <stdio.h>", "using namespace std;"];
    assert!(parse_imports("C", &lines).is_empty());
    assert!(parse_imports("C++", &lines).is_empty());
    assert!(parse_imports("Ruby", &lines).is_empty());
    assert!(parse_imports("", &lines).is_empty());
}

// =============================================================================
// 9. Normalization edge cases
// =============================================================================

#[test]
fn normalize_empty_string() {
    assert_eq!(normalize_import_target(""), "");
}

#[test]
fn normalize_whitespace_only() {
    assert_eq!(normalize_import_target("   "), "");
}

#[test]
fn normalize_deeply_nested_path() {
    assert_eq!(normalize_import_target("a/b/c/d/e/f/g"), "a");
}

#[test]
fn normalize_colon_separated() {
    assert_eq!(normalize_import_target("std::io::Read"), "std");
}

#[test]
fn normalize_dot_separated_python_style() {
    assert_eq!(normalize_import_target("os.path.join"), "os");
}
