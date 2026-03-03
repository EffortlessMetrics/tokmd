//! Error handling and edge case tests for tokmd-config.

use tokmd_settings::TomlConfig;

// ── Invalid TOML syntax ────────────────────────────────────────────

#[test]
fn parse_completely_invalid_toml() {
    let result = TomlConfig::parse("{{{{ not valid toml at all >>>>");
    assert!(result.is_err());
}

#[test]
fn parse_unclosed_bracket() {
    let result = TomlConfig::parse("[scan");
    assert!(result.is_err());
}

#[test]
fn parse_duplicate_table_headers() {
    // TOML allows redefining tables in some cases, but duplicate keys error
    let toml = r#"
[scan]
hidden = true
[scan]
hidden = false
"#;
    let result = TomlConfig::parse(toml);
    assert!(result.is_err());
}

#[test]
fn parse_invalid_value_type() {
    // `hidden` expects bool, give it a string
    let toml = r#"
[scan]
hidden = "yes"
"#;
    let result = TomlConfig::parse(toml);
    assert!(result.is_err());
}

#[test]
fn parse_invalid_nested_type() {
    // `exclude` expects array of strings, give it a number
    let toml = r#"
[scan]
exclude = 42
"#;
    let result = TomlConfig::parse(toml);
    assert!(result.is_err());
}

// ── Empty / minimal configs ────────────────────────────────────────

#[test]
fn parse_empty_string_succeeds() {
    let result = TomlConfig::parse("");
    assert!(result.is_ok());
}

#[test]
fn parse_only_whitespace_succeeds() {
    let result = TomlConfig::parse("   \n\n\t\n   ");
    assert!(result.is_ok());
}

#[test]
fn parse_only_comments_succeeds() {
    let toml = r#"
# This is a comment
# Another comment
"#;
    let result = TomlConfig::parse(toml);
    assert!(result.is_ok());
}

// ── Unknown keys (should be ignored with #[serde(default)]) ────────

#[test]
fn parse_unknown_top_level_key_is_ignored() {
    let toml = r#"
unknown_key = "value"
"#;
    // serde with deny_unknown_fields would error; without it, should succeed
    let result = TomlConfig::parse(toml);
    // Whether this succeeds or errors depends on serde configuration
    // It should succeed since TomlConfig uses #[serde(default)]
    assert!(result.is_ok());
}

#[test]
fn parse_unknown_nested_key_is_ignored() {
    let toml = r#"
[scan]
unknown_field = true
"#;
    let result = TomlConfig::parse(toml);
    assert!(result.is_ok());
}

#[test]
fn parse_extra_section_is_ignored() {
    let toml = r#"
[nonexistent_section]
key = "value"
"#;
    let result = TomlConfig::parse(toml);
    assert!(result.is_ok());
}

// ── Valid configurations ───────────────────────────────────────────

#[test]
fn parse_minimal_scan_config() {
    let toml = r#"
[scan]
hidden = true
"#;
    let config = TomlConfig::parse(toml).unwrap();
    assert_eq!(config.scan.hidden, Some(true));
}

#[test]
fn parse_scan_with_exclude_patterns() {
    let toml = r#"
[scan]
exclude = ["*.log", "target/**", "node_modules"]
"#;
    let config = TomlConfig::parse(toml).unwrap();
    let excludes = config.scan.exclude.unwrap();
    assert_eq!(excludes.len(), 3);
    assert!(excludes.contains(&"*.log".to_string()));
}

#[test]
fn parse_module_config() {
    let toml = r#"
[module]
roots = ["src", "lib"]
depth = 3
"#;
    let config = TomlConfig::parse(toml).unwrap();
    assert_eq!(config.module.depth, Some(3));
    assert_eq!(config.module.roots.as_ref().unwrap().len(), 2);
}

#[test]
fn parse_export_config() {
    let toml = r#"
[export]
min_code = 10
max_rows = 500
redact = "paths"
"#;
    let config = TomlConfig::parse(toml).unwrap();
    assert_eq!(config.export.min_code, Some(10));
    assert_eq!(config.export.max_rows, Some(500));
    assert_eq!(config.export.redact.as_deref(), Some("paths"));
}

#[test]
fn parse_gate_config() {
    let toml = r#"
[gate]
policy = "policy.toml"
baseline = "baseline.json"
preset = "health"
"#;
    let config = TomlConfig::parse(toml).unwrap();
    assert_eq!(config.gate.policy.as_deref(), Some("policy.toml"));
    assert_eq!(config.gate.baseline.as_deref(), Some("baseline.json"));
}

#[test]
fn parse_full_config_all_sections() {
    let toml = r#"
[scan]
hidden = false
exclude = ["*.tmp"]

[module]
depth = 2

[export]
min_code = 1

[analyze]
preset = "health"

[context]
budget = "128k"

[badge]
metric = "lines"

[gate]
policy = "gate.toml"
"#;
    let config = TomlConfig::parse(toml).unwrap();
    assert_eq!(config.scan.hidden, Some(false));
    assert_eq!(config.module.depth, Some(2));
    assert_eq!(config.export.min_code, Some(1));
    assert_eq!(config.analyze.preset.as_deref(), Some("health"));
    assert_eq!(config.context.budget.as_deref(), Some("128k"));
    assert_eq!(config.badge.metric.as_deref(), Some("lines"));
    assert_eq!(config.gate.policy.as_deref(), Some("gate.toml"));
}

// ── Large / extreme values ─────────────────────────────────────────

#[test]
fn parse_very_large_depth_value() {
    let toml = r#"
[module]
depth = 999999999
"#;
    let config = TomlConfig::parse(toml).unwrap();
    assert_eq!(config.module.depth, Some(999999999));
}

#[test]
fn parse_zero_values() {
    let toml = r#"
[export]
min_code = 0
max_rows = 0
"#;
    let config = TomlConfig::parse(toml).unwrap();
    assert_eq!(config.export.min_code, Some(0));
    assert_eq!(config.export.max_rows, Some(0));
}

#[test]
fn parse_empty_exclude_list() {
    let toml = r#"
[scan]
exclude = []
"#;
    let config = TomlConfig::parse(toml).unwrap();
    assert!(config.scan.exclude.unwrap().is_empty());
}

#[test]
fn parse_unicode_in_string_values() {
    let toml = r#"
[analyze]
preset = "héàlth"
"#;
    let config = TomlConfig::parse(toml).unwrap();
    assert_eq!(config.analyze.preset.as_deref(), Some("héàlth"));
}

// ── File I/O errors ────────────────────────────────────────────────

#[test]
fn from_file_nonexistent_path_returns_error() {
    let result = TomlConfig::from_file(std::path::Path::new("/nonexistent/tokmd.toml"));
    assert!(result.is_err());
}

#[test]
fn from_file_valid_config_round_trips() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(
        tmp.path(),
        r#"
[scan]
hidden = true

[module]
depth = 5
"#,
    )
    .unwrap();
    let config = TomlConfig::from_file(tmp.path()).unwrap();
    assert_eq!(config.scan.hidden, Some(true));
    assert_eq!(config.module.depth, Some(5));
}

// ── View profiles ──────────────────────────────────────────────────

#[test]
fn parse_view_profiles() {
    let toml = r#"
[view.compact]
format = "tsv"
top = 10
files = false

[view.detailed]
format = "md"
top = 50
files = true
"#;
    let config = TomlConfig::parse(toml).unwrap();
    assert_eq!(config.view.len(), 2);
    assert!(config.view.contains_key("compact"));
    assert!(config.view.contains_key("detailed"));
}

#[test]
fn parse_empty_view_profiles() {
    let toml = "";
    let config = TomlConfig::parse(toml).unwrap();
    assert!(config.view.is_empty());
}

// ── Negative numbers ───────────────────────────────────────────────

#[test]
fn parse_negative_depth_is_type_error() {
    let toml = r#"
[module]
depth = -1
"#;
    // usize cannot be negative, so this should fail
    let result = TomlConfig::parse(toml);
    assert!(result.is_err());
}
