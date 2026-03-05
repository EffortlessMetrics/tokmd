//! Feature-stability tests for WASM readiness seams.
//!
//! These tests verify that tokmd-settings works correctly WITHOUT optional
//! features. They must NOT use `#[cfg(feature = ...)]` guards.

use tokmd_settings::*;

// ── Default construction ──────────────────────────────────────────────

#[test]
fn scan_options_default_construction() {
    let opts = ScanOptions::default();
    assert!(opts.excluded.is_empty());
    assert!(!opts.hidden);
    assert!(!opts.no_ignore);
    assert!(!opts.no_ignore_parent);
    assert!(!opts.treat_doc_strings_as_comments);
}

#[test]
fn scan_settings_default_construction() {
    let s = ScanSettings::default();
    assert!(s.paths.is_empty());
}

#[test]
fn lang_settings_default_values() {
    let s = LangSettings::default();
    assert_eq!(s.top, 0);
    assert!(!s.files);
    assert!(s.redact.is_none());
}

#[test]
fn module_settings_default_values() {
    let s = ModuleSettings::default();
    assert_eq!(s.top, 0);
    assert_eq!(s.module_depth, 2);
    assert!(!s.module_roots.is_empty());
}

#[test]
fn export_settings_default_values() {
    let s = ExportSettings::default();
    assert_eq!(s.min_code, 0);
    assert_eq!(s.max_rows, 0);
    assert!(s.meta);
    assert!(s.strip_prefix.is_none());
}

#[test]
fn analyze_settings_default_values() {
    let s = AnalyzeSettings::default();
    assert_eq!(s.preset, "receipt");
    assert!(s.window.is_none());
}

#[test]
fn diff_settings_default_construction() {
    let s = DiffSettings::default();
    assert!(s.from.is_empty());
    assert!(s.to.is_empty());
}

#[test]
fn cockpit_settings_default_values() {
    let s = CockpitSettings::default();
    assert_eq!(s.base, "main");
    assert_eq!(s.head, "HEAD");
    assert_eq!(s.range_mode, "two-dot");
}

// ── Serde roundtrips ──────────────────────────────────────────────────

#[test]
fn scan_options_serde_roundtrip() {
    let opts = ScanOptions {
        excluded: vec!["target".into()],
        hidden: true,
        ..ScanOptions::default()
    };
    let json = serde_json::to_string(&opts).unwrap();
    let restored: ScanOptions = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.excluded, vec!["target".to_string()]);
    assert!(restored.hidden);
}

#[test]
fn lang_settings_serde_roundtrip() {
    let s = LangSettings::default();
    let json = serde_json::to_string(&s).unwrap();
    let restored: LangSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.top, s.top);
    assert_eq!(restored.files, s.files);
}

#[test]
fn toml_config_default_construction() {
    let cfg = TomlConfig::default();
    let toml_str = toml::to_string(&cfg).unwrap();
    let restored: TomlConfig = toml::from_str(&toml_str).unwrap();
    // Round-trip through TOML produces equivalent defaults
    assert_eq!(
        serde_json::to_string(&cfg).unwrap(),
        serde_json::to_string(&restored).unwrap()
    );
}

#[test]
fn toml_config_parse_empty_string() {
    let cfg = TomlConfig::parse("").unwrap();
    // Parsing empty TOML yields defaults
    assert!(cfg.scan.paths.is_none());
}
