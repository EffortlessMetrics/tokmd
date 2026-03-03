//! Error boundary tests for tokmd-config.
//!
//! Tests malformed input, unknown fields, empty configs, invalid values,
//! and missing config file scenarios.

use tokmd_config::TomlConfig;

// ── Malformed TOML ────────────────────────────────────────────────────

#[test]
fn malformed_toml_missing_closing_bracket() {
    let bad = "[scan\nhidden = true";
    let result = TomlConfig::parse(bad);
    assert!(result.is_err(), "unclosed bracket should fail");
}

#[test]
fn malformed_toml_duplicate_key() {
    let bad = "[scan]\nhidden = true\nhidden = false";
    let result = TomlConfig::parse(bad);
    assert!(result.is_err(), "duplicate key should fail");
}

#[test]
fn malformed_toml_bare_value() {
    let bad = "this is not valid toml at all";
    let result = TomlConfig::parse(bad);
    assert!(result.is_err(), "random text should fail");
}

#[test]
fn malformed_toml_invalid_utf8_sequence() {
    // Valid UTF-8 but nonsensical TOML structure
    let bad = "[[[\n\n]]]";
    let result = TomlConfig::parse(bad);
    assert!(result.is_err(), "triple-bracket garbage should fail");
}

// ── Unknown fields ────────────────────────────────────────────────────

#[test]
fn unknown_top_level_field_is_silently_accepted() {
    // TomlConfig uses #[serde(default)] without deny_unknown_fields,
    // so unknown sections are silently ignored by the toml crate.
    let toml = r#"
[scan]
hidden = true

[completely_unknown_section]
foo = "bar"
"#;
    let result = TomlConfig::parse(toml);
    assert!(result.is_ok(), "unknown top-level section is tolerated");
    // Known field should still be parsed
    assert_eq!(result.unwrap().scan.hidden, Some(true));
}

#[test]
fn unknown_nested_field_is_silently_accepted() {
    let toml = r#"
[scan]
hidden = true
nonexistent_flag = true
"#;
    let result = TomlConfig::parse(toml);
    assert!(
        result.is_ok(),
        "unknown nested field is tolerated by serde(default)"
    );
    assert_eq!(result.unwrap().scan.hidden, Some(true));
}

// ── Empty config ──────────────────────────────────────────────────────

#[test]
fn empty_string_parses_to_defaults() {
    let result = TomlConfig::parse("").unwrap();
    assert!(result.scan.hidden.is_none());
    assert!(result.scan.exclude.is_none());
    assert!(result.module.depth.is_none());
    assert!(result.analyze.preset.is_none());
}

#[test]
fn whitespace_only_parses_to_defaults() {
    let result = TomlConfig::parse("   \n\n  \t  ").unwrap();
    assert!(result.scan.paths.is_none());
}

// ── Invalid value types ──────────────────────────────────────────────

#[test]
fn scan_hidden_wrong_type_string_instead_of_bool() {
    let toml = r#"
[scan]
hidden = "yes"
"#;
    let result = TomlConfig::parse(toml);
    assert!(result.is_err(), "string for bool field should fail");
}

#[test]
fn module_depth_negative_number() {
    // usize can't be negative; TOML parser should reject
    let toml = r#"
[module]
depth = -1
"#;
    let result = TomlConfig::parse(toml);
    assert!(result.is_err(), "negative usize should fail");
}

#[test]
fn module_depth_float_instead_of_int() {
    let toml = r#"
[module]
depth = 3.5
"#;
    let result = TomlConfig::parse(toml);
    assert!(result.is_err(), "float for usize field should fail");
}

#[test]
fn export_min_code_string_instead_of_number() {
    let toml = r#"
[export]
min_code = "ten"
"#;
    let result = TomlConfig::parse(toml);
    assert!(result.is_err(), "string for usize field should fail");
}

#[test]
fn analyze_window_boolean_instead_of_number() {
    let toml = r#"
[analyze]
window = true
"#;
    let result = TomlConfig::parse(toml);
    assert!(result.is_err(), "bool for usize field should fail");
}

// ── Missing config file ──────────────────────────────────────────────

#[test]
fn from_file_nonexistent_returns_io_error() {
    let path = std::path::Path::new("/tmp/tokmd-absolutely-does-not-exist-12345.toml");
    let result = TomlConfig::from_file(path);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
}

#[test]
fn from_file_malformed_content_returns_invalid_data() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bad.toml");
    std::fs::write(&path, "[[[invalid").unwrap();
    let result = TomlConfig::from_file(&path);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::InvalidData);
}

// ── Gate rules in config ─────────────────────────────────────────────

#[test]
fn gate_rule_missing_required_name_field() {
    let toml = r#"
[gate]
[[gate.rules]]
pointer = "/tokens"
op = "lte"
value = 1000
"#;
    let result = TomlConfig::parse(toml);
    assert!(result.is_err(), "gate rule without name should fail");
}

#[test]
fn gate_rule_missing_required_pointer_field() {
    let toml = r#"
[gate]
[[gate.rules]]
name = "check"
op = "lte"
value = 1000
"#;
    let result = TomlConfig::parse(toml);
    assert!(result.is_err(), "gate rule without pointer should fail");
}

// ── Partial valid config ─────────────────────────────────────────────

#[test]
fn partial_config_only_scan_section() {
    let toml = r#"
[scan]
hidden = true
no_ignore = true
"#;
    let cfg = TomlConfig::parse(toml).unwrap();
    assert_eq!(cfg.scan.hidden, Some(true));
    assert_eq!(cfg.scan.no_ignore, Some(true));
    // Other sections should be defaults
    assert!(cfg.module.depth.is_none());
    assert!(cfg.analyze.preset.is_none());
}

#[test]
fn scan_exclude_wrong_type_string_instead_of_array() {
    let toml = r#"
[scan]
exclude = "target"
"#;
    let result = TomlConfig::parse(toml);
    assert!(result.is_err(), "string instead of array should fail");
}
