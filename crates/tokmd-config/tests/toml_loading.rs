//! Tests for TOML file loading workflows and profile override merging.
//!
//! Covers: minimal config, full config file I/O, profile discovery,
//! and multi-profile file scenarios.

use std::io::Write;
use tempfile::{NamedTempFile, TempDir};
use tokmd_config::{TomlConfig, ViewProfile};

// =========================================================================
// Minimal TOML loading
// =========================================================================

#[test]
fn load_minimal_toml_with_single_section() {
    let toml_str = "[scan]\nhidden = true\n";
    let config = TomlConfig::parse(toml_str).expect("valid minimal TOML");
    assert_eq!(config.scan.hidden, Some(true));
    // All other sections should remain at defaults
    assert_eq!(config.module.depth, None);
    assert_eq!(config.export.format, None);
    assert_eq!(config.analyze.preset, None);
    assert_eq!(config.context.budget, None);
    assert_eq!(config.badge.metric, None);
    assert_eq!(config.gate.fail_fast, None);
    assert!(config.view.is_empty());
}

#[test]
fn load_toml_with_only_view_profiles() {
    let toml_str = r#"
[view.alpha]
format = "json"

[view.beta]
format = "tsv"
top = 5
"#;
    let config = TomlConfig::parse(toml_str).expect("valid");
    assert_eq!(config.view.len(), 2);
    assert_eq!(
        config.view.get("alpha").unwrap().format,
        Some("json".to_string())
    );
    assert_eq!(config.view.get("beta").unwrap().top, Some(5));
    // Core sections untouched
    assert_eq!(config.scan.hidden, None);
}

// =========================================================================
// Full TOML with all fields populated (smoke test for completeness)
// =========================================================================

#[test]
fn load_fully_populated_toml_file() {
    let toml_str = r#"
[scan]
hidden = true
no_ignore = true
no_ignore_parent = true
no_ignore_dot = true
no_ignore_vcs = true
doc_comments = true
paths = ["src", "crates", "tests"]
exclude = ["target", "node_modules", "**/*.min.js"]
config = "none"

[module]
roots = ["crates", "packages", "libs"]
depth = 3
children = "collapse"

[export]
format = "csv"
min_code = 10
max_rows = 5000
redact = "all"
children = "separate"

[analyze]
preset = "deep"
window = 256000
format = "json"
git = true
max_files = 2000
max_bytes = 50000000
max_file_bytes = 500000
max_commits = 1000
max_commit_files = 100
granularity = "file"

[context]
budget = "1m"
strategy = "spread"
rank_by = "hotspot"
output = "bundle"
compress = true

[badge]
metric = "hotspot"

[gate]
fail_fast = true

[[gate.rules]]
name = "code-limit"
pointer = "/summary/total_code"
op = "lt"
value = 200000

[[gate.ratchet]]
pointer = "/complexity/avg_cyclomatic"
max_increase_pct = 5.0
level = "error"
description = "Complexity guard"

[view.llm]
format = "json"
redact = "all"
top = 25
compress = true
preset = "health"
window = 128000
budget = "64k"
strategy = "greedy"
rank_by = "code"
output = "list"
metric = "tokens"
"#;

    let config = TomlConfig::parse(toml_str).expect("fully populated TOML");

    // Spot-check every section is populated
    assert_eq!(config.scan.hidden, Some(true));
    assert_eq!(config.scan.no_ignore, Some(true));
    assert_eq!(config.scan.no_ignore_parent, Some(true));
    assert_eq!(config.scan.no_ignore_dot, Some(true));
    assert_eq!(config.scan.no_ignore_vcs, Some(true));
    assert_eq!(config.scan.doc_comments, Some(true));
    assert_eq!(config.scan.paths.as_ref().unwrap().len(), 3);
    assert_eq!(config.scan.exclude.as_ref().unwrap().len(), 3);
    assert_eq!(config.scan.config, Some("none".to_string()));

    assert_eq!(config.module.roots.as_ref().unwrap().len(), 3);
    assert_eq!(config.module.depth, Some(3));
    assert_eq!(config.module.children, Some("collapse".to_string()));

    assert_eq!(config.export.format, Some("csv".to_string()));
    assert_eq!(config.export.min_code, Some(10));
    assert_eq!(config.export.max_rows, Some(5000));
    assert_eq!(config.export.redact, Some("all".to_string()));
    assert_eq!(config.export.children, Some("separate".to_string()));

    assert_eq!(config.analyze.preset, Some("deep".to_string()));
    assert_eq!(config.analyze.window, Some(256000));
    assert_eq!(config.analyze.format, Some("json".to_string()));
    assert_eq!(config.analyze.git, Some(true));
    assert_eq!(config.analyze.max_files, Some(2000));
    assert_eq!(config.analyze.max_bytes, Some(50_000_000));
    assert_eq!(config.analyze.max_file_bytes, Some(500_000));
    assert_eq!(config.analyze.max_commits, Some(1000));
    assert_eq!(config.analyze.max_commit_files, Some(100));
    assert_eq!(config.analyze.granularity, Some("file".to_string()));

    assert_eq!(config.context.budget, Some("1m".to_string()));
    assert_eq!(config.context.strategy, Some("spread".to_string()));
    assert_eq!(config.context.rank_by, Some("hotspot".to_string()));
    assert_eq!(config.context.output, Some("bundle".to_string()));
    assert_eq!(config.context.compress, Some(true));

    assert_eq!(config.badge.metric, Some("hotspot".to_string()));

    assert_eq!(config.gate.fail_fast, Some(true));
    assert!(config.gate.rules.is_some());
    assert!(config.gate.ratchet.is_some());

    let llm = config.view.get("llm").expect("llm profile");
    assert_eq!(llm.format, Some("json".to_string()));
    assert_eq!(llm.redact, Some("all".to_string()));
    assert_eq!(llm.top, Some(25));
    assert_eq!(llm.compress, Some(true));
    assert_eq!(llm.preset, Some("health".to_string()));
    assert_eq!(llm.window, Some(128000));
    assert_eq!(llm.budget, Some("64k".to_string()));
    assert_eq!(llm.strategy, Some("greedy".to_string()));
    assert_eq!(llm.rank_by, Some("code".to_string()));
    assert_eq!(llm.output, Some("list".to_string()));
    assert_eq!(llm.metric, Some("tokens".to_string()));
}

// =========================================================================
// File I/O: write to temp file, load back
// =========================================================================

#[test]
fn load_from_temp_file_and_verify() {
    let content = r#"
[module]
roots = ["apps", "services"]
depth = 4

[view.custom]
format = "md"
top = 15
"#;

    let mut tmp = NamedTempFile::new().expect("create temp");
    tmp.write_all(content.as_bytes()).expect("write");

    let config = TomlConfig::from_file(tmp.path()).expect("load from file");
    assert_eq!(
        config.module.roots,
        Some(vec!["apps".to_string(), "services".to_string()])
    );
    assert_eq!(config.module.depth, Some(4));
    assert_eq!(
        config.view.get("custom").unwrap().format,
        Some("md".to_string())
    );
}

#[test]
fn load_from_directory_file() {
    let dir = TempDir::new().expect("create temp dir");
    let file_path = dir.path().join("tokmd.toml");
    std::fs::write(
        &file_path,
        "[scan]\nhidden = true\n[badge]\nmetric = \"bytes\"\n",
    )
    .expect("write file");

    let config = TomlConfig::from_file(&file_path).expect("load");
    assert_eq!(config.scan.hidden, Some(true));
    assert_eq!(config.badge.metric, Some("bytes".to_string()));
}

// =========================================================================
// Profile override merging
// =========================================================================

/// Simulates how CLI code merges a ViewProfile onto a ViewProfile base,
/// keeping profile values where Some, falling through to base otherwise.
fn merge(base: &ViewProfile, overlay: &ViewProfile) -> ViewProfile {
    ViewProfile {
        format: overlay.format.clone().or_else(|| base.format.clone()),
        top: overlay.top.or(base.top),
        files: overlay.files.or(base.files),
        module_roots: overlay
            .module_roots
            .clone()
            .or_else(|| base.module_roots.clone()),
        module_depth: overlay.module_depth.or(base.module_depth),
        min_code: overlay.min_code.or(base.min_code),
        max_rows: overlay.max_rows.or(base.max_rows),
        redact: overlay.redact.clone().or_else(|| base.redact.clone()),
        meta: overlay.meta.or(base.meta),
        children: overlay.children.clone().or_else(|| base.children.clone()),
        preset: overlay.preset.clone().or_else(|| base.preset.clone()),
        window: overlay.window.or(base.window),
        budget: overlay.budget.clone().or_else(|| base.budget.clone()),
        strategy: overlay.strategy.clone().or_else(|| base.strategy.clone()),
        rank_by: overlay.rank_by.clone().or_else(|| base.rank_by.clone()),
        output: overlay.output.clone().or_else(|| base.output.clone()),
        compress: overlay.compress.or(base.compress),
        metric: overlay.metric.clone().or_else(|| base.metric.clone()),
    }
}

#[test]
fn three_profile_cascade_from_toml() {
    // Load three profiles from TOML and cascade: base → org → user
    let toml_str = r#"
[view.base]
format = "md"
top = 0
budget = "128k"

[view.org]
format = "json"
min_code = 10
strategy = "spread"

[view.user]
top = 50
compress = true
"#;
    let config = TomlConfig::parse(toml_str).expect("valid");
    let base = config.view.get("base").unwrap();
    let org = config.view.get("org").unwrap();
    let user = config.view.get("user").unwrap();

    let after_org = merge(base, org);
    let final_result = merge(&after_org, user);

    // user overrides
    assert_eq!(final_result.top, Some(50));
    assert_eq!(final_result.compress, Some(true));
    // org overrides
    assert_eq!(final_result.format, Some("json".to_string()));
    assert_eq!(final_result.min_code, Some(10));
    assert_eq!(final_result.strategy, Some("spread".to_string()));
    // base survives
    assert_eq!(final_result.budget, Some("128k".to_string()));
}

#[test]
fn profile_override_replaces_not_appends_module_roots() {
    let base = ViewProfile {
        module_roots: Some(vec!["crates".to_string()]),
        ..Default::default()
    };
    let overlay = ViewProfile {
        module_roots: Some(vec!["packages".to_string(), "libs".to_string()]),
        ..Default::default()
    };

    let merged = merge(&base, &overlay);
    // overlay completely replaces base, doesn't merge arrays
    assert_eq!(
        merged.module_roots,
        Some(vec!["packages".to_string(), "libs".to_string()])
    );
}

// =========================================================================
// Error messages from invalid TOML files
// =========================================================================

#[test]
fn invalid_toml_from_file_wraps_in_io_error() {
    let mut tmp = NamedTempFile::new().expect("create temp");
    tmp.write_all(b"[[[broken").expect("write");
    let err = TomlConfig::from_file(tmp.path()).unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
}

#[test]
fn parse_duplicate_section_last_wins_or_errors() {
    // TOML spec: duplicate keys within a table are errors.
    // But duplicate table headers may be merged or error depending on parser.
    let toml_str = r#"
[scan]
hidden = true

[scan]
no_ignore = true
"#;
    // toml crate typically errors on duplicate table headers
    let result = TomlConfig::parse(toml_str);
    assert!(result.is_err(), "Duplicate [scan] should be an error");
}
