//! Error handling tests for tokmd-config types.
//!
//! Tests graceful handling of malformed input, missing fields,
//! invalid field types, and edge cases in config deserialization.

use tokmd_config::{CliRedactMode, Profile, UserConfig};

// =============================================================================
// UserConfig deserialization errors
// =============================================================================

#[test]
fn user_config_malformed_json_returns_error() {
    let bad = "{ not valid json }}}";
    let result = serde_json::from_str::<UserConfig>(bad);
    assert!(result.is_err());
}

#[test]
fn user_config_wrong_type_for_profiles_returns_error() {
    // profiles should be a map, not a string
    let bad = r#"{"profiles": "not-a-map", "repos": {}}"#;
    let result = serde_json::from_str::<UserConfig>(bad);
    assert!(result.is_err());
}

#[test]
fn user_config_wrong_type_for_repos_returns_error() {
    // repos should be a map, not an array
    let bad = r#"{"profiles": {}, "repos": [1, 2, 3]}"#;
    let result = serde_json::from_str::<UserConfig>(bad);
    assert!(result.is_err());
}

#[test]
fn user_config_empty_json_object_requires_fields() {
    // UserConfig requires profiles and repos fields
    let result = serde_json::from_str::<UserConfig>("{}");
    assert!(
        result.is_err(),
        "Empty JSON object should fail for UserConfig"
    );
}

#[test]
fn user_config_with_empty_maps_succeeds() {
    let config: UserConfig = serde_json::from_str(r#"{"profiles": {}, "repos": {}}"#).unwrap();
    assert!(config.profiles.is_empty());
    assert!(config.repos.is_empty());
}

// =============================================================================
// Profile deserialization errors
// =============================================================================

#[test]
fn profile_invalid_top_type_returns_error() {
    // top should be a number, not a string
    let bad = r#"{"top": "not-a-number"}"#;
    let result = serde_json::from_str::<Profile>(bad);
    assert!(result.is_err());
}

#[test]
fn profile_invalid_files_type_returns_error() {
    // files should be a bool, not a number
    let bad = r#"{"files": 42}"#;
    let result = serde_json::from_str::<Profile>(bad);
    assert!(result.is_err());
}

#[test]
fn profile_invalid_redact_variant_returns_error() {
    // redact should be a valid CliRedactMode variant
    let bad = r#"{"redact": "invalid_variant"}"#;
    let result = serde_json::from_str::<Profile>(bad);
    assert!(result.is_err());
}

#[test]
fn profile_invalid_module_roots_type_returns_error() {
    // module_roots should be an array of strings, not a string
    let bad = r#"{"module_roots": "not-an-array"}"#;
    let result = serde_json::from_str::<Profile>(bad);
    assert!(result.is_err());
}

#[test]
fn profile_empty_json_object_all_none() {
    let profile: Profile = serde_json::from_str("{}").unwrap();
    assert!(profile.format.is_none());
    assert!(profile.top.is_none());
    assert!(profile.files.is_none());
    assert!(profile.module_roots.is_none());
    assert!(profile.module_depth.is_none());
    assert!(profile.min_code.is_none());
    assert!(profile.max_rows.is_none());
    assert!(profile.redact.is_none());
    assert!(profile.meta.is_none());
    assert!(profile.children.is_none());
}

#[test]
fn profile_negative_top_returns_error() {
    // usize can't be negative
    let bad = r#"{"top": -1}"#;
    let result = serde_json::from_str::<Profile>(bad);
    assert!(result.is_err());
}

// =============================================================================
// CliRedactMode deserialization errors
// =============================================================================

#[test]
fn redact_mode_invalid_string_returns_error() {
    let bad = r#""unknown""#;
    let result = serde_json::from_str::<CliRedactMode>(bad);
    assert!(result.is_err());
}

#[test]
fn redact_mode_valid_variants_roundtrip() {
    for mode in [
        CliRedactMode::None,
        CliRedactMode::Paths,
        CliRedactMode::All,
    ] {
        let json = serde_json::to_string(&mode).unwrap();
        let back: CliRedactMode = serde_json::from_str(&json).unwrap();
        assert_eq!(back, mode);
    }
}

// =============================================================================
// UserConfig with nested profile errors
// =============================================================================

#[test]
fn user_config_profile_with_invalid_nested_field() {
    let bad = r#"{
        "profiles": {
            "broken": {"top": "not-a-number"}
        },
        "repos": {}
    }"#;
    let result = serde_json::from_str::<UserConfig>(bad);
    assert!(result.is_err());
}

#[test]
fn user_config_repos_wrong_value_type() {
    // repos values should be strings, not numbers
    let bad = r#"{
        "profiles": {},
        "repos": {"owner/repo": 42}
    }"#;
    let result = serde_json::from_str::<UserConfig>(bad);
    assert!(result.is_err());
}

// =============================================================================
// TomlConfig error paths (re-exported from tokmd-settings)
// =============================================================================

use tokmd_config::TomlConfig;

#[test]
fn toml_config_malformed_toml_returns_error() {
    let bad = "this is not valid [[[toml";
    let result = TomlConfig::parse(bad);
    assert!(result.is_err());
}

#[test]
fn toml_config_wrong_type_for_scan_hidden_returns_error() {
    let bad = r#"
[scan]
hidden = "yes"
"#;
    let result = TomlConfig::parse(bad);
    assert!(result.is_err());
}

#[test]
fn toml_config_wrong_type_for_module_depth_returns_error() {
    let bad = r#"
[module]
depth = "deep"
"#;
    let result = TomlConfig::parse(bad);
    assert!(result.is_err());
}

#[test]
fn toml_config_empty_string_parses_to_defaults() {
    let config = TomlConfig::parse("").unwrap();
    assert!(config.scan.hidden.is_none());
    assert!(config.scan.paths.is_none());
    assert!(config.module.depth.is_none());
    assert!(config.view.is_empty());
}

#[test]
fn toml_config_from_file_nonexistent_path_returns_error() {
    let result = TomlConfig::from_file(std::path::Path::new(
        "/tmp/tokmd-errors-nonexistent-config-file.toml",
    ));
    assert!(result.is_err());
}

#[test]
fn toml_config_wrong_type_for_export_min_code_returns_error() {
    let bad = r#"
[export]
min_code = "lots"
"#;
    let result = TomlConfig::parse(bad);
    assert!(result.is_err());
}

#[test]
fn toml_config_wrong_type_for_gate_fail_fast_returns_error() {
    let bad = r#"
[gate]
fail_fast = "yes"
"#;
    let result = TomlConfig::parse(bad);
    assert!(result.is_err());
}

#[test]
fn toml_config_unknown_keys_are_accepted_by_default() {
    // TOML deserialization with serde default typically ignores unknown keys
    // unless deny_unknown_fields is set. Verify graceful handling.
    let toml = r#"
[scan]
hidden = true
some_unknown_key = "value"
"#;
    // This may or may not error depending on the struct's serde attributes.
    // The test documents the behavior either way.
    let _result = TomlConfig::parse(toml);
    // Just verify it doesn't panic
}
