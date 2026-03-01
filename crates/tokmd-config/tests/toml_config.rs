//! Tests for TomlConfig parsing and loading.
//!
//! These tests ensure that configuration values are correctly parsed from TOML,
//! not just that parsing succeeds.

use std::io::Write;
use tempfile::NamedTempFile;
use tokmd_config::TomlConfig;

#[test]
fn test_parse_returns_correct_values() {
    let toml_str = r#"
[scan]
hidden = true
no_ignore = true

[module]
depth = 3
roots = ["src", "lib"]

[export]
min_code = 10
max_rows = 100
redact = "paths"

[analyze]
preset = "deep"
window = 128000
max_files = 500
max_bytes = 1000000
max_commits = 200

[context]
budget = "64k"
strategy = "spread"
compress = true

[badge]
metric = "tokens"

[gate]
fail_fast = true
"#;

    let config = TomlConfig::parse(toml_str).expect("valid TOML");

    // Verify scan settings
    assert_eq!(config.scan.hidden, Some(true));
    assert_eq!(config.scan.no_ignore, Some(true));

    // Verify module settings
    assert_eq!(config.module.depth, Some(3));
    assert_eq!(
        config.module.roots,
        Some(vec!["src".to_string(), "lib".to_string()])
    );

    // Verify export settings
    assert_eq!(config.export.min_code, Some(10));
    assert_eq!(config.export.max_rows, Some(100));
    assert_eq!(config.export.redact, Some("paths".to_string()));

    // Verify analyze settings
    assert_eq!(config.analyze.preset, Some("deep".to_string()));
    assert_eq!(config.analyze.window, Some(128000));
    assert_eq!(config.analyze.max_files, Some(500));
    assert_eq!(config.analyze.max_bytes, Some(1000000));
    assert_eq!(config.analyze.max_commits, Some(200));

    // Verify context settings
    assert_eq!(config.context.budget, Some("64k".to_string()));
    assert_eq!(config.context.strategy, Some("spread".to_string()));
    assert_eq!(config.context.compress, Some(true));

    // Verify badge settings
    assert_eq!(config.badge.metric, Some("tokens".to_string()));

    // Verify gate settings
    assert_eq!(config.gate.fail_fast, Some(true));
}

#[test]
fn test_parse_invalid_toml_returns_error() {
    let invalid_toml = r#"
[scan
hidden = true
"#;

    let result = TomlConfig::parse(invalid_toml);
    assert!(result.is_err(), "Should fail to parse invalid TOML");
}

#[test]
fn test_from_file_returns_correct_values() {
    let toml_content = r#"
[scan]
hidden = true
paths = ["src", "tests"]

[module]
depth = 4
children = "collapse"

[export]
format = "jsonl"
min_code = 5
"#;

    let mut temp_file = NamedTempFile::new().expect("create temp file");
    temp_file
        .write_all(toml_content.as_bytes())
        .expect("write temp file");

    let config = TomlConfig::from_file(temp_file.path()).expect("load config from file");

    // Verify scan settings
    assert_eq!(config.scan.hidden, Some(true));
    assert_eq!(
        config.scan.paths,
        Some(vec!["src".to_string(), "tests".to_string()])
    );

    // Verify module settings
    assert_eq!(config.module.depth, Some(4));
    assert_eq!(config.module.children, Some("collapse".to_string()));

    // Verify export settings
    assert_eq!(config.export.format, Some("jsonl".to_string()));
    assert_eq!(config.export.min_code, Some(5));
}

#[test]
fn test_from_file_nonexistent_returns_error() {
    let result = TomlConfig::from_file(std::path::Path::new("/nonexistent/path/tokmd.toml"));
    assert!(result.is_err(), "Should fail for nonexistent file");
}

#[test]
fn test_from_file_invalid_toml_returns_error() {
    let invalid_toml = r#"
[scan
hidden = true
"#;

    let mut temp_file = NamedTempFile::new().expect("create temp file");
    temp_file
        .write_all(invalid_toml.as_bytes())
        .expect("write temp file");

    let result = TomlConfig::from_file(temp_file.path());
    assert!(
        result.is_err(),
        "Should fail to parse invalid TOML from file"
    );
}

#[test]
fn test_parse_view_profiles() {
    let toml_str = r#"
[view.llm_safe]
format = "json"
redact = "all"
top = 10

[view.ci]
format = "tsv"
min_code = 1
"#;

    let config = TomlConfig::parse(toml_str).expect("valid TOML");

    // Verify llm_safe profile
    let llm_profile = config
        .view
        .get("llm_safe")
        .expect("llm_safe profile exists");
    assert_eq!(llm_profile.format, Some("json".to_string()));
    assert_eq!(llm_profile.redact, Some("all".to_string()));
    assert_eq!(llm_profile.top, Some(10));

    // Verify ci profile
    let ci_profile = config.view.get("ci").expect("ci profile exists");
    assert_eq!(ci_profile.format, Some("tsv".to_string()));
    assert_eq!(ci_profile.min_code, Some(1));
}

#[test]
fn test_parse_gate_rules() {
    let toml_str = r#"
[gate]
fail_fast = true

[[gate.rules]]
name = "max-lines"
pointer = "/summary/total_code"
op = "lt"
value = 100000

[[gate.rules]]
name = "no-todos"
pointer = "/health/todo_density"
op = "lt"
value = 0.01
level = "warn"
message = "Too many TODOs"
"#;

    let config = TomlConfig::parse(toml_str).expect("valid TOML");

    assert_eq!(config.gate.fail_fast, Some(true));

    let rules = config.gate.rules.expect("rules exist");
    assert_eq!(rules.len(), 2);

    assert_eq!(rules[0].name, "max-lines");
    assert_eq!(rules[0].pointer, "/summary/total_code");
    assert_eq!(rules[0].op, "lt");

    assert_eq!(rules[1].name, "no-todos");
    assert_eq!(rules[1].level, Some("warn".to_string()));
    assert_eq!(rules[1].message, Some("Too many TODOs".to_string()));
}
