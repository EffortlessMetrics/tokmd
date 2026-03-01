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

// =========================================================================
// Empty / minimal TOML
// =========================================================================

#[test]
fn test_parse_empty_string_yields_defaults() {
    let config = TomlConfig::parse("").expect("empty TOML is valid");
    assert_eq!(config.scan.hidden, None);
    assert_eq!(config.scan.no_ignore, None);
    assert_eq!(config.module.depth, None);
    assert_eq!(config.module.roots, None);
    assert_eq!(config.export.min_code, None);
    assert_eq!(config.analyze.preset, None);
    assert_eq!(config.context.budget, None);
    assert_eq!(config.badge.metric, None);
    assert_eq!(config.gate.fail_fast, None);
    assert!(config.view.is_empty());
}

#[test]
fn test_parse_empty_sections() {
    let toml_str = r#"
[scan]
[module]
[export]
[analyze]
[context]
[badge]
[gate]
"#;
    let config = TomlConfig::parse(toml_str).expect("valid TOML");
    assert_eq!(config.scan.hidden, None);
    assert_eq!(config.module.depth, None);
    assert_eq!(config.export.format, None);
    assert_eq!(config.analyze.window, None);
    assert_eq!(config.context.strategy, None);
    assert_eq!(config.badge.metric, None);
    assert_eq!(config.gate.fail_fast, None);
}

// =========================================================================
// Partial sections — only some fields in each section
// =========================================================================

#[test]
fn test_parse_scan_partial_only_hidden() {
    let config = TomlConfig::parse("[scan]\nhidden = true").expect("valid");
    assert_eq!(config.scan.hidden, Some(true));
    assert_eq!(config.scan.no_ignore, None);
    assert_eq!(config.scan.no_ignore_parent, None);
    assert_eq!(config.scan.no_ignore_dot, None);
    assert_eq!(config.scan.no_ignore_vcs, None);
    assert_eq!(config.scan.doc_comments, None);
    assert_eq!(config.scan.paths, None);
    assert_eq!(config.scan.exclude, None);
}

#[test]
fn test_parse_scan_all_ignore_flags() {
    let toml_str = r#"
[scan]
no_ignore = true
no_ignore_parent = true
no_ignore_dot = true
no_ignore_vcs = true
"#;
    let config = TomlConfig::parse(toml_str).expect("valid");
    assert_eq!(config.scan.no_ignore, Some(true));
    assert_eq!(config.scan.no_ignore_parent, Some(true));
    assert_eq!(config.scan.no_ignore_dot, Some(true));
    assert_eq!(config.scan.no_ignore_vcs, Some(true));
}

#[test]
fn test_parse_scan_exclude_patterns() {
    let toml_str = r#"
[scan]
exclude = ["target", "node_modules", "**/*.min.js"]
"#;
    let config = TomlConfig::parse(toml_str).expect("valid");
    assert_eq!(
        config.scan.exclude,
        Some(vec![
            "target".to_string(),
            "node_modules".to_string(),
            "**/*.min.js".to_string()
        ])
    );
}

#[test]
fn test_parse_scan_doc_comments() {
    let config = TomlConfig::parse("[scan]\ndoc_comments = true").expect("valid");
    assert_eq!(config.scan.doc_comments, Some(true));
}

// =========================================================================
// Analyze config — all fields
// =========================================================================

#[test]
fn test_parse_analyze_all_fields() {
    let toml_str = r#"
[analyze]
preset = "risk"
window = 200000
format = "json"
git = true
max_files = 1000
max_bytes = 5000000
max_file_bytes = 100000
max_commits = 500
max_commit_files = 50
granularity = "file"
"#;
    let config = TomlConfig::parse(toml_str).expect("valid");
    assert_eq!(config.analyze.preset, Some("risk".to_string()));
    assert_eq!(config.analyze.window, Some(200_000));
    assert_eq!(config.analyze.format, Some("json".to_string()));
    assert_eq!(config.analyze.git, Some(true));
    assert_eq!(config.analyze.max_files, Some(1000));
    assert_eq!(config.analyze.max_bytes, Some(5_000_000));
    assert_eq!(config.analyze.max_file_bytes, Some(100_000));
    assert_eq!(config.analyze.max_commits, Some(500));
    assert_eq!(config.analyze.max_commit_files, Some(50));
    assert_eq!(config.analyze.granularity, Some("file".to_string()));
}

// =========================================================================
// Context config — all fields
// =========================================================================

#[test]
fn test_parse_context_all_fields() {
    let toml_str = r#"
[context]
budget = "256k"
strategy = "greedy"
rank_by = "hotspot"
output = "bundle"
compress = false
"#;
    let config = TomlConfig::parse(toml_str).expect("valid");
    assert_eq!(config.context.budget, Some("256k".to_string()));
    assert_eq!(config.context.strategy, Some("greedy".to_string()));
    assert_eq!(config.context.rank_by, Some("hotspot".to_string()));
    assert_eq!(config.context.output, Some("bundle".to_string()));
    assert_eq!(config.context.compress, Some(false));
}

// =========================================================================
// Export config — all fields
// =========================================================================

#[test]
fn test_parse_export_all_fields() {
    let toml_str = r#"
[export]
min_code = 0
max_rows = 5000
redact = "all"
format = "csv"
children = "collapse"
"#;
    let config = TomlConfig::parse(toml_str).expect("valid");
    assert_eq!(config.export.min_code, Some(0));
    assert_eq!(config.export.max_rows, Some(5000));
    assert_eq!(config.export.redact, Some("all".to_string()));
    assert_eq!(config.export.format, Some("csv".to_string()));
    assert_eq!(config.export.children, Some("collapse".to_string()));
}

// =========================================================================
// Module config — all fields
// =========================================================================

#[test]
fn test_parse_module_all_fields() {
    let toml_str = r#"
[module]
roots = ["crates", "packages", "libs"]
depth = 2
children = "separate"
"#;
    let config = TomlConfig::parse(toml_str).expect("valid");
    assert_eq!(
        config.module.roots,
        Some(vec![
            "crates".to_string(),
            "packages".to_string(),
            "libs".to_string()
        ])
    );
    assert_eq!(config.module.depth, Some(2));
    assert_eq!(config.module.children, Some("separate".to_string()));
}

// =========================================================================
// Gate rules — ratchet rules
// =========================================================================

#[test]
fn test_parse_gate_ratchet_rules() {
    let toml_str = r#"
[gate]

[[gate.ratchet]]
pointer = "/complexity/avg_cyclomatic"
max_increase_pct = 5.0
level = "error"
description = "Cyclomatic complexity must not increase more than 5%"

[[gate.ratchet]]
pointer = "/summary/total_code"
max_value = 100000.0
level = "warn"
"#;
    let config = TomlConfig::parse(toml_str).expect("valid");
    let ratchet = config.gate.ratchet.expect("ratchet rules exist");
    assert_eq!(ratchet.len(), 2);

    assert_eq!(ratchet[0].pointer, "/complexity/avg_cyclomatic");
    assert_eq!(ratchet[0].max_increase_pct, Some(5.0));
    assert_eq!(ratchet[0].level, Some("error".to_string()));
    assert_eq!(
        ratchet[0].description,
        Some("Cyclomatic complexity must not increase more than 5%".to_string())
    );

    assert_eq!(ratchet[1].pointer, "/summary/total_code");
    assert_eq!(ratchet[1].max_value, Some(100_000.0));
    assert_eq!(ratchet[1].level, Some("warn".to_string()));
    assert_eq!(ratchet[1].description, None);
}

#[test]
fn test_parse_gate_rules_with_negate_and_values() {
    let toml_str = r#"
[[gate.rules]]
name = "allowed-languages"
pointer = "/summary/languages"
op = "in"
values = ["Rust", "Python", "Go"]

[[gate.rules]]
name = "not-fortran"
pointer = "/summary/primary_language"
op = "eq"
value = "Fortran"
negate = true
level = "error"
message = "Fortran detected — please migrate"
"#;
    let config = TomlConfig::parse(toml_str).expect("valid");
    let rules = config.gate.rules.expect("rules exist");
    assert_eq!(rules.len(), 2);

    // First rule: in operator with values
    assert_eq!(rules[0].name, "allowed-languages");
    assert_eq!(rules[0].op, "in");
    let values = rules[0].values.as_ref().expect("values exist");
    assert_eq!(values.len(), 3);
    assert!(!rules[0].negate);

    // Second rule: negate
    assert_eq!(rules[1].name, "not-fortran");
    assert!(rules[1].negate);
    assert_eq!(
        rules[1].message,
        Some("Fortran detected — please migrate".to_string())
    );
}

#[test]
fn test_parse_gate_combined_rules_and_ratchet() {
    let toml_str = r#"
[gate]
fail_fast = false

[[gate.rules]]
name = "max-code"
pointer = "/summary/total_code"
op = "lt"
value = 50000

[[gate.ratchet]]
pointer = "/complexity/avg_cyclomatic"
max_increase_pct = 10.0
"#;
    let config = TomlConfig::parse(toml_str).expect("valid");
    assert_eq!(config.gate.fail_fast, Some(false));
    assert_eq!(config.gate.rules.as_ref().map(|r| r.len()), Some(1));
    assert_eq!(config.gate.ratchet.as_ref().map(|r| r.len()), Some(1));
}

// =========================================================================
// View profiles — comprehensive
// =========================================================================

#[test]
fn test_parse_view_profile_all_fields() {
    let toml_str = r#"
[view.full]
format = "json"
top = 20
files = true
module_roots = ["src", "lib"]
module_depth = 3
min_code = 5
max_rows = 500
redact = "paths"
meta = false
children = "collapse"
preset = "deep"
window = 128000
budget = "256k"
strategy = "spread"
rank_by = "churn"
output = "bundle"
compress = true
metric = "lines"
"#;
    let config = TomlConfig::parse(toml_str).expect("valid");
    let p = config.view.get("full").expect("full profile exists");
    assert_eq!(p.format, Some("json".to_string()));
    assert_eq!(p.top, Some(20));
    assert_eq!(p.files, Some(true));
    assert_eq!(
        p.module_roots,
        Some(vec!["src".to_string(), "lib".to_string()])
    );
    assert_eq!(p.module_depth, Some(3));
    assert_eq!(p.min_code, Some(5));
    assert_eq!(p.max_rows, Some(500));
    assert_eq!(p.redact, Some("paths".to_string()));
    assert_eq!(p.meta, Some(false));
    assert_eq!(p.children, Some("collapse".to_string()));
    assert_eq!(p.preset, Some("deep".to_string()));
    assert_eq!(p.window, Some(128_000));
    assert_eq!(p.budget, Some("256k".to_string()));
    assert_eq!(p.strategy, Some("spread".to_string()));
    assert_eq!(p.rank_by, Some("churn".to_string()));
    assert_eq!(p.output, Some("bundle".to_string()));
    assert_eq!(p.compress, Some(true));
    assert_eq!(p.metric, Some("lines".to_string()));
}

#[test]
fn test_parse_multiple_view_profiles_sorted() {
    let toml_str = r#"
[view.z_last]
format = "tsv"

[view.a_first]
format = "json"

[view.m_middle]
format = "md"
"#;
    let config = TomlConfig::parse(toml_str).expect("valid");
    assert_eq!(config.view.len(), 3);
    // BTreeMap guarantees sorted iteration order
    let keys: Vec<&String> = config.view.keys().collect();
    assert_eq!(keys, vec!["a_first", "m_middle", "z_last"]);
}

#[test]
fn test_parse_view_profile_empty() {
    let toml_str = r#"
[view.empty]
"#;
    let config = TomlConfig::parse(toml_str).expect("valid");
    let p = config.view.get("empty").expect("empty profile exists");
    assert_eq!(p.format, None);
    assert_eq!(p.top, None);
    assert_eq!(p.files, None);
}

// =========================================================================
// Unknown fields are silently ignored (serde default behavior)
// =========================================================================

#[test]
fn test_parse_unknown_top_level_key_is_silently_ignored() {
    // With #[serde(default)] on TomlConfig, unknown top-level keys are
    // silently ignored by the TOML deserializer.
    let toml_str = r#"
totally_unknown_key = "value"
"#;
    let result = TomlConfig::parse(toml_str);
    assert!(
        result.is_ok(),
        "Unknown top-level keys should be silently ignored"
    );
}

// =========================================================================
// Edge cases — zero / boundary values
// =========================================================================

#[test]
fn test_parse_zero_values() {
    let toml_str = r#"
[export]
min_code = 0
max_rows = 0

[analyze]
window = 0
max_files = 0
max_bytes = 0

[view.zero_top]
top = 0
"#;
    let config = TomlConfig::parse(toml_str).expect("valid");
    assert_eq!(config.export.min_code, Some(0));
    assert_eq!(config.export.max_rows, Some(0));
    assert_eq!(config.analyze.window, Some(0));
    assert_eq!(config.analyze.max_files, Some(0));
    assert_eq!(config.analyze.max_bytes, Some(0));
    assert_eq!(config.view.get("zero_top").unwrap().top, Some(0));
}

#[test]
fn test_parse_large_values() {
    let toml_str = r#"
[analyze]
max_bytes = 9999999999999
window = 999999999
"#;
    let config = TomlConfig::parse(toml_str).expect("valid");
    assert_eq!(config.analyze.max_bytes, Some(9_999_999_999_999));
    assert_eq!(config.analyze.window, Some(999_999_999));
}

#[test]
fn test_parse_empty_arrays() {
    let toml_str = r#"
[scan]
paths = []
exclude = []

[module]
roots = []
"#;
    let config = TomlConfig::parse(toml_str).expect("valid");
    assert_eq!(config.scan.paths, Some(vec![]));
    assert_eq!(config.scan.exclude, Some(vec![]));
    assert_eq!(config.module.roots, Some(vec![]));
}

#[test]
fn test_parse_empty_strings() {
    let toml_str = r#"
[export]
format = ""
redact = ""

[context]
budget = ""
"#;
    let config = TomlConfig::parse(toml_str).expect("valid");
    assert_eq!(config.export.format, Some(String::new()));
    assert_eq!(config.export.redact, Some(String::new()));
    assert_eq!(config.context.budget, Some(String::new()));
}

// =========================================================================
// Full realistic config (kitchen sink)
// =========================================================================

#[test]
fn test_parse_full_realistic_config() {
    let toml_str = r#"
[scan]
hidden = false
no_ignore = false
no_ignore_vcs = false
paths = ["src", "crates"]
exclude = ["target", "node_modules"]
doc_comments = true

[module]
roots = ["crates", "packages"]
depth = 2
children = "separate"

[export]
format = "jsonl"
min_code = 1
max_rows = 10000
redact = "none"
children = "collapse"

[analyze]
preset = "health"
window = 128000
format = "md"
git = false
max_files = 2000
max_bytes = 10000000
max_commits = 500
granularity = "module"

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

[view.llm]
format = "json"
redact = "all"
top = 25

[view.ci]
format = "tsv"
min_code = 1
max_rows = 50
"#;
    let config = TomlConfig::parse(toml_str).expect("valid full config");

    // Spot-check various sections
    assert_eq!(config.scan.hidden, Some(false));
    assert_eq!(config.scan.doc_comments, Some(true));
    assert_eq!(config.module.depth, Some(2));
    assert_eq!(config.export.format, Some("jsonl".to_string()));
    assert_eq!(config.analyze.preset, Some("health".to_string()));
    assert_eq!(config.analyze.git, Some(false));
    assert_eq!(config.context.budget, Some("128k".to_string()));
    assert_eq!(config.badge.metric, Some("lines".to_string()));
    assert_eq!(config.gate.fail_fast, Some(true));
    assert_eq!(config.gate.rules.as_ref().map(|r| r.len()), Some(1));
    assert_eq!(config.gate.ratchet.as_ref().map(|r| r.len()), Some(1));
    assert_eq!(config.view.len(), 2);
    assert_eq!(
        config.view.get("llm").unwrap().redact,
        Some("all".to_string())
    );
}

// =========================================================================
// TOML round-trip: parse → serialize → re-parse yields same values
// =========================================================================

#[test]
fn test_toml_roundtrip_via_json() {
    let toml_str = r#"
[scan]
hidden = true

[module]
depth = 3
roots = ["crates"]

[export]
min_code = 10

[view.ci]
format = "tsv"
"#;
    let config1 = TomlConfig::parse(toml_str).expect("parse");
    let json = serde_json::to_string(&config1).expect("to json");
    let config2: TomlConfig = serde_json::from_str(&json).expect("from json");

    assert_eq!(config1.scan.hidden, config2.scan.hidden);
    assert_eq!(config1.module.depth, config2.module.depth);
    assert_eq!(config1.module.roots, config2.module.roots);
    assert_eq!(config1.export.min_code, config2.export.min_code);
    assert_eq!(
        config1.view.get("ci").unwrap().format,
        config2.view.get("ci").unwrap().format
    );
}

// =========================================================================
// from_file with realistic temp file
// =========================================================================

#[test]
fn test_from_file_empty_file_yields_defaults() {
    let mut temp = NamedTempFile::new().expect("create temp");
    temp.write_all(b"").expect("write");
    let config = TomlConfig::from_file(temp.path()).expect("load empty");
    assert_eq!(config.scan.hidden, None);
    assert!(config.view.is_empty());
}

#[test]
fn test_from_file_utf8_content() {
    // TOML requires quoting non-ASCII keys
    let toml_content = "[view.\"日本語\"]\nformat = \"json\"\n";
    let mut temp = NamedTempFile::new().expect("create temp");
    temp.write_all(toml_content.as_bytes()).expect("write");
    let config = TomlConfig::from_file(temp.path()).expect("load utf8");
    assert!(config.view.contains_key("日本語"));
    assert_eq!(
        config.view.get("日本語").unwrap().format,
        Some("json".to_string())
    );
}
