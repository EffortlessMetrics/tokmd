//! Tests for TomlConfig parsing and loading.
//!
//! These tests ensure that configuration values are correctly parsed from TOML,
//! not just that parsing succeeds.

use std::io::Write;
use tempfile::NamedTempFile;
use tokmd_config::{
    AnalysisPreset, ChildIncludeMode, ChildrenMode, ExportFormat, RedactMode, TableFormat,
    TomlConfig,
};

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

// =========================================================================
// TOML round-trip: parse → serialize to TOML → re-parse yields same values
// =========================================================================

#[test]
fn test_toml_full_roundtrip_via_toml() {
    let original = r#"
[scan]
hidden = true
no_ignore = false
paths = ["src", "lib"]
exclude = ["target"]

[module]
roots = ["crates", "packages"]
depth = 2
children = "separate"

[export]
min_code = 5
max_rows = 1000
redact = "paths"
format = "csv"
children = "collapse"

[analyze]
preset = "risk"
window = 200000
format = "json"
git = true
max_files = 500
max_bytes = 5000000
max_file_bytes = 100000
max_commits = 300
max_commit_files = 50
granularity = "file"

[context]
budget = "256k"
strategy = "spread"
rank_by = "hotspot"
output = "bundle"
compress = true

[badge]
metric = "tokens"

[gate]
fail_fast = true

[view.ci]
format = "tsv"
top = 20
min_code = 1
"#;

    let config1 = TomlConfig::parse(original).expect("parse original");
    let toml_string = toml::to_string(&config1).expect("serialize to TOML");
    let config2 = TomlConfig::parse(&toml_string).expect("re-parse serialized");

    // Verify all sections round-trip
    assert_eq!(config1.scan.hidden, config2.scan.hidden);
    assert_eq!(config1.scan.no_ignore, config2.scan.no_ignore);
    assert_eq!(config1.scan.paths, config2.scan.paths);
    assert_eq!(config1.scan.exclude, config2.scan.exclude);
    assert_eq!(config1.module.roots, config2.module.roots);
    assert_eq!(config1.module.depth, config2.module.depth);
    assert_eq!(config1.module.children, config2.module.children);
    assert_eq!(config1.export.min_code, config2.export.min_code);
    assert_eq!(config1.export.max_rows, config2.export.max_rows);
    assert_eq!(config1.export.redact, config2.export.redact);
    assert_eq!(config1.export.format, config2.export.format);
    assert_eq!(config1.export.children, config2.export.children);
    assert_eq!(config1.analyze.preset, config2.analyze.preset);
    assert_eq!(config1.analyze.window, config2.analyze.window);
    assert_eq!(config1.analyze.format, config2.analyze.format);
    assert_eq!(config1.analyze.git, config2.analyze.git);
    assert_eq!(config1.analyze.max_files, config2.analyze.max_files);
    assert_eq!(config1.analyze.max_bytes, config2.analyze.max_bytes);
    assert_eq!(
        config1.analyze.max_file_bytes,
        config2.analyze.max_file_bytes
    );
    assert_eq!(config1.analyze.max_commits, config2.analyze.max_commits);
    assert_eq!(
        config1.analyze.max_commit_files,
        config2.analyze.max_commit_files
    );
    assert_eq!(config1.analyze.granularity, config2.analyze.granularity);
    assert_eq!(config1.context.budget, config2.context.budget);
    assert_eq!(config1.context.strategy, config2.context.strategy);
    assert_eq!(config1.context.rank_by, config2.context.rank_by);
    assert_eq!(config1.context.output, config2.context.output);
    assert_eq!(config1.context.compress, config2.context.compress);
    assert_eq!(config1.badge.metric, config2.badge.metric);
    assert_eq!(config1.gate.fail_fast, config2.gate.fail_fast);
    assert_eq!(
        config1.view.get("ci").unwrap().format,
        config2.view.get("ci").unwrap().format
    );
    assert_eq!(
        config1.view.get("ci").unwrap().top,
        config2.view.get("ci").unwrap().top
    );
    assert_eq!(
        config1.view.get("ci").unwrap().min_code,
        config2.view.get("ci").unwrap().min_code
    );
}

#[test]
fn test_toml_roundtrip_empty_config() {
    let config1 = TomlConfig::parse("").expect("parse empty");
    let toml_string = toml::to_string(&config1).expect("serialize");
    let config2 = TomlConfig::parse(&toml_string).expect("re-parse");
    assert_eq!(config1.scan.hidden, config2.scan.hidden);
    assert!(config2.view.is_empty());
}

#[test]
fn test_toml_roundtrip_gate_rules() {
    let original = r#"
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
description = "Complexity guard"
"#;
    let config1 = TomlConfig::parse(original).expect("parse");
    let toml_string = toml::to_string(&config1).expect("serialize");
    let config2 = TomlConfig::parse(&toml_string).expect("re-parse");

    let rules1 = config1.gate.rules.as_ref().unwrap();
    let rules2 = config2.gate.rules.as_ref().unwrap();
    assert_eq!(rules1.len(), rules2.len());
    assert_eq!(rules1[0].name, rules2[0].name);
    assert_eq!(rules1[0].pointer, rules2[0].pointer);
    assert_eq!(rules1[0].op, rules2[0].op);

    let ratchet1 = config1.gate.ratchet.as_ref().unwrap();
    let ratchet2 = config2.gate.ratchet.as_ref().unwrap();
    assert_eq!(ratchet1.len(), ratchet2.len());
    assert_eq!(ratchet1[0].pointer, ratchet2[0].pointer);
    assert_eq!(ratchet1[0].max_increase_pct, ratchet2[0].max_increase_pct);
    assert_eq!(ratchet1[0].level, ratchet2[0].level);
    assert_eq!(ratchet1[0].description, ratchet2[0].description);
}

// =========================================================================
// Invalid TOML produces descriptive errors
// =========================================================================

#[test]
fn test_invalid_toml_error_contains_location() {
    let bad = "[scan\nhidden = true";
    let err = TomlConfig::parse(bad).unwrap_err();
    let msg = err.to_string();
    // Error should mention the location of the problem
    assert!(
        msg.contains("expected") || msg.contains("line") || msg.contains("1"),
        "Error should be descriptive: {msg}"
    );
}

#[test]
fn test_type_mismatch_error_mentions_field() {
    let bad = "[scan]\nhidden = 42";
    let err = TomlConfig::parse(bad).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("invalid type") || msg.contains("expected"),
        "Error should mention type mismatch: {msg}"
    );
}

#[test]
fn test_from_file_invalid_toml_error_propagates() {
    let bad = "{{{{not valid toml";
    let mut temp = NamedTempFile::new().expect("temp");
    temp.write_all(bad.as_bytes()).expect("write");
    let err = TomlConfig::from_file(temp.path()).unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
}

// =========================================================================
// All analysis presets are recognized in TOML
// =========================================================================

#[test]
fn test_all_analysis_preset_strings_parse_in_toml() {
    let presets = [
        "receipt",
        "health",
        "risk",
        "supply",
        "architecture",
        "topics",
        "security",
        "identity",
        "git",
        "deep",
        "fun",
    ];
    for preset in presets {
        let toml_str = format!("[analyze]\npreset = \"{}\"", preset);
        let config = TomlConfig::parse(&toml_str)
            .unwrap_or_else(|e| panic!("preset '{}' should parse: {}", preset, e));
        assert_eq!(config.analyze.preset, Some(preset.to_string()));
    }
}

#[test]
fn test_all_analysis_presets_roundtrip_via_clap_value() {
    // Verify each AnalysisPreset variant serializes to a known preset string
    let presets = [
        (AnalysisPreset::Receipt, "receipt"),
        (AnalysisPreset::Health, "health"),
        (AnalysisPreset::Risk, "risk"),
        (AnalysisPreset::Supply, "supply"),
        (AnalysisPreset::Architecture, "architecture"),
        (AnalysisPreset::Topics, "topics"),
        (AnalysisPreset::Security, "security"),
        (AnalysisPreset::Identity, "identity"),
        (AnalysisPreset::Git, "git"),
        (AnalysisPreset::Deep, "deep"),
        (AnalysisPreset::Fun, "fun"),
    ];
    for (variant, expected_str) in presets {
        let json = serde_json::to_string(&variant).expect("serialize");
        assert_eq!(
            json,
            format!("\"{}\"", expected_str),
            "AnalysisPreset::{:?} should serialize to \"{}\"",
            variant,
            expected_str
        );
    }
}

// =========================================================================
// Children mode parsing in TOML
// =========================================================================

#[test]
fn test_children_mode_collapse_in_module_toml() {
    let config = TomlConfig::parse("[module]\nchildren = \"collapse\"").expect("valid");
    assert_eq!(config.module.children, Some("collapse".to_string()));
}

#[test]
fn test_children_mode_separate_in_module_toml() {
    let config = TomlConfig::parse("[module]\nchildren = \"separate\"").expect("valid");
    assert_eq!(config.module.children, Some("separate".to_string()));
}

#[test]
fn test_children_mode_collapse_in_export_toml() {
    let config = TomlConfig::parse("[export]\nchildren = \"collapse\"").expect("valid");
    assert_eq!(config.export.children, Some("collapse".to_string()));
}

#[test]
fn test_children_mode_separate_in_export_toml() {
    let config = TomlConfig::parse("[export]\nchildren = \"separate\"").expect("valid");
    assert_eq!(config.export.children, Some("separate".to_string()));
}

#[test]
fn test_children_mode_enum_serde_roundtrip() {
    // Verify ChildrenMode enum values round-trip through JSON
    let collapse: ChildrenMode = serde_json::from_str("\"collapse\"").expect("parse collapse");
    assert_eq!(collapse, ChildrenMode::Collapse);

    let separate: ChildrenMode = serde_json::from_str("\"separate\"").expect("parse separate");
    assert_eq!(separate, ChildrenMode::Separate);
}

#[test]
fn test_child_include_mode_enum_serde_roundtrip() {
    let sep: ChildIncludeMode = serde_json::from_str("\"separate\"").expect("parse");
    assert_eq!(sep, ChildIncludeMode::Separate);

    let parents: ChildIncludeMode = serde_json::from_str("\"parents-only\"").expect("parse");
    assert_eq!(parents, ChildIncludeMode::ParentsOnly);
}

// =========================================================================
// Format parsing in TOML
// =========================================================================

#[test]
fn test_all_export_format_strings_in_toml() {
    let formats = ["json", "jsonl", "csv", "cyclonedx"];
    for fmt in formats {
        let toml_str = format!("[export]\nformat = \"{}\"", fmt);
        let config = TomlConfig::parse(&toml_str)
            .unwrap_or_else(|e| panic!("format '{}' should parse: {}", fmt, e));
        assert_eq!(config.export.format, Some(fmt.to_string()));
    }
}

#[test]
fn test_export_format_enum_serde() {
    let cases = [
        ("\"csv\"", ExportFormat::Csv),
        ("\"jsonl\"", ExportFormat::Jsonl),
        ("\"json\"", ExportFormat::Json),
        ("\"cyclonedx\"", ExportFormat::Cyclonedx),
    ];
    for (json_str, expected) in cases {
        let parsed: ExportFormat = serde_json::from_str(json_str)
            .unwrap_or_else(|e| panic!("{} should parse: {}", json_str, e));
        assert_eq!(parsed, expected);
    }
}

#[test]
fn test_table_format_enum_serde() {
    let cases = [
        ("\"md\"", TableFormat::Md),
        ("\"tsv\"", TableFormat::Tsv),
        ("\"json\"", TableFormat::Json),
    ];
    for (json_str, expected) in cases {
        let parsed: TableFormat = serde_json::from_str(json_str)
            .unwrap_or_else(|e| panic!("{} should parse: {}", json_str, e));
        assert_eq!(parsed, expected);
    }
}

#[test]
fn test_redact_mode_enum_serde() {
    let cases = [
        ("\"none\"", RedactMode::None),
        ("\"paths\"", RedactMode::Paths),
        ("\"all\"", RedactMode::All),
    ];
    for (json_str, expected) in cases {
        let parsed: RedactMode = serde_json::from_str(json_str)
            .unwrap_or_else(|e| panic!("{} should parse: {}", json_str, e));
        assert_eq!(parsed, expected);
    }
}

#[test]
fn test_all_context_strategy_values_in_toml() {
    for strategy in ["greedy", "spread"] {
        let toml_str = format!("[context]\nstrategy = \"{}\"", strategy);
        let config = TomlConfig::parse(&toml_str).expect("valid");
        assert_eq!(config.context.strategy, Some(strategy.to_string()));
    }
}

#[test]
fn test_all_rank_by_values_in_toml() {
    for rank_by in ["code", "tokens", "churn", "hotspot"] {
        let toml_str = format!("[context]\nrank_by = \"{}\"", rank_by);
        let config = TomlConfig::parse(&toml_str).expect("valid");
        assert_eq!(config.context.rank_by, Some(rank_by.to_string()));
    }
}

#[test]
fn test_all_context_output_values_in_toml() {
    for output in ["list", "bundle", "json"] {
        let toml_str = format!("[context]\noutput = \"{}\"", output);
        let config = TomlConfig::parse(&toml_str).expect("valid");
        assert_eq!(config.context.output, Some(output.to_string()));
    }
}

#[test]
fn test_analyze_granularity_values_in_toml() {
    for granularity in ["module", "file"] {
        let toml_str = format!("[analyze]\ngranularity = \"{}\"", granularity);
        let config = TomlConfig::parse(&toml_str).expect("valid");
        assert_eq!(config.analyze.granularity, Some(granularity.to_string()));
    }
}

// =========================================================================
// Default values for settings structs (via tokmd-settings)
// =========================================================================

#[test]
fn test_toml_config_default_all_sections_none() {
    let config = TomlConfig::default();
    assert_eq!(config.scan.hidden, None);
    assert_eq!(config.scan.no_ignore, None);
    assert_eq!(config.scan.paths, None);
    assert_eq!(config.scan.exclude, None);
    assert_eq!(config.scan.config, None);
    assert_eq!(config.module.roots, None);
    assert_eq!(config.module.depth, None);
    assert_eq!(config.module.children, None);
    assert_eq!(config.export.min_code, None);
    assert_eq!(config.export.max_rows, None);
    assert_eq!(config.export.redact, None);
    assert_eq!(config.export.format, None);
    assert_eq!(config.export.children, None);
    assert_eq!(config.analyze.preset, None);
    assert_eq!(config.analyze.window, None);
    assert_eq!(config.analyze.format, None);
    assert_eq!(config.analyze.git, None);
    assert_eq!(config.analyze.max_files, None);
    assert_eq!(config.context.budget, None);
    assert_eq!(config.context.strategy, None);
    assert_eq!(config.context.rank_by, None);
    assert_eq!(config.context.output, None);
    assert_eq!(config.context.compress, None);
    assert_eq!(config.badge.metric, None);
    assert_eq!(config.gate.fail_fast, None);
    assert!(config.gate.rules.is_none());
    assert!(config.gate.ratchet.is_none());
    assert!(config.view.is_empty());
}

#[test]
fn test_settings_defaults_are_sensible() {
    use tokmd_settings::{
        AnalyzeSettings, CockpitSettings, ExportSettings, LangSettings, ModuleSettings,
    };

    let lang = LangSettings::default();
    assert_eq!(lang.top, 0); // show all
    assert!(!lang.files);
    assert_eq!(lang.children, ChildrenMode::Collapse);
    assert_eq!(lang.redact, None);

    let module = ModuleSettings::default();
    assert_eq!(module.top, 0);
    assert_eq!(module.module_roots, vec!["crates", "packages"]);
    assert_eq!(module.module_depth, 2);
    assert_eq!(module.children, ChildIncludeMode::Separate);

    let export = ExportSettings::default();
    assert_eq!(export.format, ExportFormat::Jsonl);
    assert_eq!(export.min_code, 0);
    assert_eq!(export.max_rows, 0);
    assert_eq!(export.redact, RedactMode::None);
    assert!(export.meta);

    let analyze = AnalyzeSettings::default();
    assert_eq!(analyze.preset, "receipt");
    assert_eq!(analyze.granularity, "module");
    assert_eq!(analyze.git, None);
    assert_eq!(analyze.window, None);

    let cockpit = CockpitSettings::default();
    assert_eq!(cockpit.base, "main");
    assert_eq!(cockpit.head, "HEAD");
    assert_eq!(cockpit.range_mode, "two-dot");
    assert_eq!(cockpit.baseline, None);
}
