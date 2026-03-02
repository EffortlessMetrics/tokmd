//! Extended TOML parsing tests for tokmd-config.
//!
//! Tests cover profile inheritance semantics, default values, invalid config
//! handling, and edge cases in tokmd.toml parsing.

use std::io::Write;
use tempfile::NamedTempFile;
use tokmd_config::TomlConfig;

// ── 1. Profile inheritance: view profile inherits from section defaults ──

#[test]
fn view_profile_fields_override_section_defaults_independently() {
    let toml_str = r#"
[export]
format = "jsonl"
min_code = 10

[view.ci]
format = "csv"
min_code = 0

[view.llm]
format = "json"
"#;
    let config = TomlConfig::parse(toml_str).expect("valid TOML");

    // Export section keeps its values
    assert_eq!(config.export.format, Some("jsonl".to_string()));
    assert_eq!(config.export.min_code, Some(10));

    // ci profile has its own values
    let ci = config.view.get("ci").expect("ci profile");
    assert_eq!(ci.format, Some("csv".to_string()));
    assert_eq!(ci.min_code, Some(0));

    // llm profile only has format, no min_code
    let llm = config.view.get("llm").expect("llm profile");
    assert_eq!(llm.format, Some("json".to_string()));
    assert_eq!(llm.min_code, None);
}

// ── 2. Multiple profiles with distinct settings ─────────────────────────

#[test]
fn multiple_view_profiles_are_independent() {
    let toml_str = r#"
[view.quick]
format = "md"
top = 5
compress = false

[view.deep]
format = "json"
top = 100
preset = "deep"
window = 256000
compress = true
"#;
    let config = TomlConfig::parse(toml_str).expect("valid TOML");

    let quick = config.view.get("quick").unwrap();
    let deep = config.view.get("deep").unwrap();

    assert_eq!(quick.format, Some("md".to_string()));
    assert_eq!(quick.top, Some(5));
    assert_eq!(quick.compress, Some(false));
    assert_eq!(quick.preset, None);

    assert_eq!(deep.format, Some("json".to_string()));
    assert_eq!(deep.top, Some(100));
    assert_eq!(deep.compress, Some(true));
    assert_eq!(deep.preset, Some("deep".to_string()));
    assert_eq!(deep.window, Some(256000));
}

// ── 3. Default values: empty TOML yields all-None config ────────────────

#[test]
fn empty_toml_yields_all_none_defaults() {
    let config = TomlConfig::parse("").expect("valid");

    assert_eq!(config.scan.hidden, None);
    assert_eq!(config.scan.no_ignore, None);
    assert_eq!(config.scan.paths, None);
    assert_eq!(config.scan.exclude, None);
    assert_eq!(config.module.depth, None);
    assert_eq!(config.module.roots, None);
    assert_eq!(config.export.format, None);
    assert_eq!(config.export.min_code, None);
    assert_eq!(config.analyze.preset, None);
    assert_eq!(config.analyze.window, None);
    assert_eq!(config.context.budget, None);
    assert_eq!(config.context.strategy, None);
    assert_eq!(config.badge.metric, None);
    assert_eq!(config.gate.fail_fast, None);
    assert!(config.gate.rules.is_none());
    assert!(config.gate.ratchet.is_none());
    assert!(config.view.is_empty());
}

// ── 4. Invalid TOML syntax is rejected ──────────────────────────────────

#[test]
fn invalid_toml_syntax_returns_error() {
    let cases = [
        "[scan\nhidden = true",               // missing closing bracket
        "scan.hidden = [true, false",         // unclosed array
        "[module]\ndepth = \"not a number\"", // wrong type (string → usize)
    ];

    for case in &cases {
        let result = TomlConfig::parse(case);
        assert!(result.is_err(), "Should fail for: {}", case);
    }
}

// ── 5. Type mismatch errors ─────────────────────────────────────────────

#[test]
fn type_mismatches_return_errors() {
    // String where bool expected
    assert!(TomlConfig::parse("[scan]\nhidden = \"yes\"").is_err());
    // Bool where usize expected
    assert!(TomlConfig::parse("[module]\ndepth = true").is_err());
    // Float where usize expected
    assert!(TomlConfig::parse("[analyze]\nwindow = 3.14").is_err());
    // Array where string expected
    assert!(TomlConfig::parse("[context]\nbudget = [\"128k\"]").is_err());
    // String where array expected
    assert!(TomlConfig::parse("[module]\nroots = \"crates\"").is_err());
    // Negative where unsigned expected
    assert!(TomlConfig::parse("[export]\nmin_code = -1").is_err());
}

// ── 6. Gate rules parsing ───────────────────────────────────────────────

#[test]
fn gate_rules_with_all_fields_are_parsed_correctly() {
    let toml_str = r#"
[gate]
fail_fast = true

[[gate.rules]]
name = "max-lines"
pointer = "/summary/total_code"
op = "lt"
value = 100000
level = "error"
message = "Too many lines of code"

[[gate.rules]]
name = "lang-check"
pointer = "/summary/languages"
op = "in"
values = ["Rust", "Python"]
negate = true
"#;
    let config = TomlConfig::parse(toml_str).expect("valid");
    assert_eq!(config.gate.fail_fast, Some(true));

    let rules = config.gate.rules.unwrap();
    assert_eq!(rules.len(), 2);

    assert_eq!(rules[0].name, "max-lines");
    assert_eq!(rules[0].op, "lt");
    assert_eq!(rules[0].level, Some("error".to_string()));
    assert_eq!(rules[0].message, Some("Too many lines of code".to_string()));
    assert!(!rules[0].negate);

    assert_eq!(rules[1].name, "lang-check");
    assert!(rules[1].negate);
    assert!(rules[1].values.is_some());
}

// ── 7. Ratchet rules parsing ────────────────────────────────────────────

#[test]
fn ratchet_rules_with_mixed_constraints() {
    let toml_str = r#"
[[gate.ratchet]]
pointer = "/complexity/avg_cyclomatic"
max_increase_pct = 5.0
level = "error"
description = "Complexity must not increase by more than 5%"

[[gate.ratchet]]
pointer = "/summary/total_code"
max_value = 100000.0
level = "warn"
"#;
    let config = TomlConfig::parse(toml_str).expect("valid");
    let ratchet = config.gate.ratchet.unwrap();
    assert_eq!(ratchet.len(), 2);

    assert_eq!(ratchet[0].pointer, "/complexity/avg_cyclomatic");
    assert_eq!(ratchet[0].max_increase_pct, Some(5.0));
    assert_eq!(ratchet[0].max_value, None);
    assert_eq!(ratchet[0].level, Some("error".to_string()));

    assert_eq!(ratchet[1].pointer, "/summary/total_code");
    assert_eq!(ratchet[1].max_increase_pct, None);
    assert_eq!(ratchet[1].max_value, Some(100_000.0));
    assert_eq!(ratchet[1].level, Some("warn".to_string()));
}

// ── 8. from_file with valid and invalid files ───────────────────────────

#[test]
fn from_file_parses_valid_toml() {
    let content = r#"
[scan]
hidden = true
exclude = ["target"]

[module]
depth = 3

[view.test]
format = "json"
"#;
    let mut tmp = NamedTempFile::new().unwrap();
    tmp.write_all(content.as_bytes()).unwrap();

    let config = TomlConfig::from_file(tmp.path()).expect("load from file");
    assert_eq!(config.scan.hidden, Some(true));
    assert_eq!(config.scan.exclude, Some(vec!["target".to_string()]));
    assert_eq!(config.module.depth, Some(3));
    assert!(config.view.contains_key("test"));
}

#[test]
fn from_file_nonexistent_path_returns_error() {
    let result = TomlConfig::from_file(std::path::Path::new("/no/such/path/tokmd.toml"));
    assert!(result.is_err());
}

#[test]
fn from_file_invalid_toml_returns_error() {
    let mut tmp = NamedTempFile::new().unwrap();
    tmp.write_all(b"[broken\nhidden = true").unwrap();

    let result = TomlConfig::from_file(tmp.path());
    assert!(result.is_err());
}

// ── 9. View profiles: BTreeMap ordering ─────────────────────────────────

#[test]
fn view_profiles_are_stored_in_sorted_order() {
    let toml_str = r#"
[view.zulu]
format = "md"

[view.alpha]
format = "json"

[view.mike]
format = "tsv"
"#;
    let config = TomlConfig::parse(toml_str).expect("valid");
    let keys: Vec<&String> = config.view.keys().collect();
    assert_eq!(keys, vec!["alpha", "mike", "zulu"]);
}

// ── 10. JSON round-trip: parse → serialize → re-parse ───────────────────

#[test]
fn toml_config_roundtrips_through_json() {
    let toml_str = r#"
[scan]
hidden = true
no_ignore = false
exclude = ["dist", "*.lock"]

[module]
depth = 4
roots = ["crates", "packages"]

[export]
format = "csv"
min_code = 5
max_rows = 1000

[analyze]
preset = "risk"
window = 200000
git = true

[context]
budget = "256k"
strategy = "spread"
compress = true

[badge]
metric = "tokens"

[gate]
fail_fast = true

[view.ci]
format = "tsv"
top = 50
"#;
    let config1 = TomlConfig::parse(toml_str).expect("parse");
    let json = serde_json::to_string(&config1).expect("to json");
    let config2: TomlConfig = serde_json::from_str(&json).expect("from json");

    // Spot-check all sections survived the round-trip
    assert_eq!(config1.scan.hidden, config2.scan.hidden);
    assert_eq!(config1.scan.no_ignore, config2.scan.no_ignore);
    assert_eq!(config1.scan.exclude, config2.scan.exclude);
    assert_eq!(config1.module.depth, config2.module.depth);
    assert_eq!(config1.module.roots, config2.module.roots);
    assert_eq!(config1.export.format, config2.export.format);
    assert_eq!(config1.export.min_code, config2.export.min_code);
    assert_eq!(config1.analyze.preset, config2.analyze.preset);
    assert_eq!(config1.analyze.window, config2.analyze.window);
    assert_eq!(config1.analyze.git, config2.analyze.git);
    assert_eq!(config1.context.budget, config2.context.budget);
    assert_eq!(config1.context.strategy, config2.context.strategy);
    assert_eq!(config1.context.compress, config2.context.compress);
    assert_eq!(config1.badge.metric, config2.badge.metric);
    assert_eq!(config1.gate.fail_fast, config2.gate.fail_fast);
    assert_eq!(
        config1.view.get("ci").unwrap().format,
        config2.view.get("ci").unwrap().format,
    );
    assert_eq!(
        config1.view.get("ci").unwrap().top,
        config2.view.get("ci").unwrap().top,
    );
}

// ── 11. Unknown top-level keys are silently ignored ─────────────────────

#[test]
fn unknown_keys_are_silently_ignored() {
    let toml_str = r#"
unknown_section = "ignored"

[scan]
hidden = true
"#;
    let config = TomlConfig::parse(toml_str).expect("valid");
    assert_eq!(config.scan.hidden, Some(true));
}

// ── 12. Boundary values ─────────────────────────────────────────────────

#[test]
fn zero_and_large_values_are_accepted() {
    let toml_str = r#"
[export]
min_code = 0
max_rows = 0

[analyze]
window = 0
max_bytes = 9999999999999

[view.extremes]
top = 0
"#;
    let config = TomlConfig::parse(toml_str).expect("valid");
    assert_eq!(config.export.min_code, Some(0));
    assert_eq!(config.export.max_rows, Some(0));
    assert_eq!(config.analyze.window, Some(0));
    assert_eq!(config.analyze.max_bytes, Some(9_999_999_999_999));
    assert_eq!(config.view.get("extremes").unwrap().top, Some(0));
}

// ── 13. Full kitchen-sink config ────────────────────────────────────────

#[test]
fn full_kitchen_sink_config_parses() {
    let toml_str = r#"
[scan]
hidden = false
no_ignore = false
no_ignore_parent = true
no_ignore_dot = false
no_ignore_vcs = true
doc_comments = true
paths = ["src", "crates", "tests"]
exclude = ["target", "node_modules", "**/*.min.js"]

[module]
roots = ["crates", "packages"]
depth = 2
children = "collapse"

[export]
format = "jsonl"
min_code = 1
max_rows = 10000
redact = "paths"
children = "separate"

[analyze]
preset = "health"
window = 128000
format = "json"
git = false
max_files = 2000
max_bytes = 10000000
max_file_bytes = 100000
max_commits = 500
max_commit_files = 50
granularity = "file"

[context]
budget = "128k"
strategy = "greedy"
rank_by = "code"
output = "list"
compress = false

[badge]
metric = "lines"

[gate]
fail_fast = true

[[gate.rules]]
name = "max-lines"
pointer = "/summary/total_code"
op = "lt"
value = 100000

[[gate.ratchet]]
pointer = "/complexity/avg_cyclomatic"
max_increase_pct = 5.0
level = "error"
description = "Keep complexity in check"

[view.llm]
format = "json"
redact = "all"
top = 25
compress = true
preset = "deep"

[view.ci]
format = "tsv"
min_code = 1
max_rows = 50
"#;
    let config = TomlConfig::parse(toml_str).expect("full config");

    // Spot-check a few fields from each section
    assert_eq!(config.scan.doc_comments, Some(true));
    assert_eq!(config.scan.no_ignore_parent, Some(true));
    assert_eq!(config.module.children, Some("collapse".to_string()));
    assert_eq!(config.export.redact, Some("paths".to_string()));
    assert_eq!(config.analyze.granularity, Some("file".to_string()));
    assert_eq!(config.analyze.max_file_bytes, Some(100_000));
    assert_eq!(config.context.rank_by, Some("code".to_string()));
    assert_eq!(config.badge.metric, Some("lines".to_string()));
    assert_eq!(config.gate.rules.as_ref().map(|r| r.len()), Some(1));
    assert_eq!(config.gate.ratchet.as_ref().map(|r| r.len()), Some(1));
    assert_eq!(config.view.len(), 2);
    assert_eq!(config.view.get("llm").unwrap().compress, Some(true));
}
