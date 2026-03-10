//! Backward compatibility tests for tokmd-settings.

use tokmd_settings::{
    AnalyzeSettings, CockpitSettings, ExportSettings, LangSettings, ModuleSettings, ScanOptions,
    ScanSettings, TomlConfig,
};
use tokmd_types::{ChildIncludeMode, ChildrenMode, ConfigMode, ExportFormat, RedactMode};

// ── ScanOptions defaults ─────────────────────────────────────────────────

#[test]
fn scan_options_defaults() {
    let opts = ScanOptions::default();
    assert!(opts.excluded.is_empty());
    assert_eq!(opts.config, ConfigMode::Auto);
    assert!(!opts.hidden);
    assert!(!opts.no_ignore);
    assert!(!opts.no_ignore_parent);
    assert!(!opts.no_ignore_dot);
    assert!(!opts.no_ignore_vcs);
    assert!(!opts.treat_doc_strings_as_comments);
}

#[test]
fn scan_settings_defaults() {
    let s = ScanSettings::default();
    assert!(s.paths.is_empty());
    assert!(s.options.excluded.is_empty());
}

#[test]
fn lang_settings_defaults() {
    let l = LangSettings::default();
    assert_eq!(l.top, 0);
    assert!(!l.files);
    assert_eq!(l.children, ChildrenMode::Collapse);
    assert!(l.redact.is_none());
}

#[test]
fn module_settings_defaults() {
    let m = ModuleSettings::default();
    assert_eq!(m.top, 0);
    assert_eq!(m.module_depth, 2);
    assert_eq!(m.children, ChildIncludeMode::Separate);
    assert!(m.module_roots.contains(&"crates".to_string()));
    assert!(m.module_roots.contains(&"packages".to_string()));
}

#[test]
fn export_settings_defaults() {
    let e = ExportSettings::default();
    assert_eq!(e.format, ExportFormat::Jsonl);
    assert_eq!(e.module_depth, 2);
    assert_eq!(e.min_code, 0);
    assert_eq!(e.max_rows, 0);
    assert_eq!(e.redact, RedactMode::None);
    assert!(e.meta);
    assert!(e.strip_prefix.is_none());
}

#[test]
fn analyze_settings_defaults() {
    let a = AnalyzeSettings::default();
    assert_eq!(a.preset, "receipt");
    assert_eq!(a.granularity, "module");
    assert!(a.window.is_none());
    assert!(a.git.is_none());
    assert!(a.max_files.is_none());
}

#[test]
fn cockpit_settings_defaults() {
    let c = CockpitSettings::default();
    assert_eq!(c.base, "main");
    assert_eq!(c.head, "HEAD");
    assert_eq!(c.range_mode, "two-dot");
    assert!(c.baseline.is_none());
}

// ── JSON serialization round-trips ───────────────────────────────────────

#[test]
fn scan_options_json_roundtrip() {
    let opts = ScanOptions {
        excluded: vec!["target".into(), "node_modules".into()],
        config: ConfigMode::None,
        hidden: true,
        no_ignore: false,
        no_ignore_parent: true,
        no_ignore_dot: false,
        no_ignore_vcs: true,
        treat_doc_strings_as_comments: true,
    };
    let json = serde_json::to_string(&opts).unwrap();
    let back: ScanOptions = serde_json::from_str(&json).unwrap();
    assert_eq!(back.excluded, opts.excluded);
    assert!(back.hidden);
    assert!(back.no_ignore_parent);
    assert!(back.no_ignore_vcs);
    assert!(back.treat_doc_strings_as_comments);
}

#[test]
fn analyze_settings_json_roundtrip() {
    let s = AnalyzeSettings {
        preset: "deep".into(),
        window: Some(128_000),
        git: Some(true),
        max_files: Some(500),
        max_bytes: Some(10_000_000),
        max_file_bytes: Some(1_000_000),
        max_commits: Some(100),
        max_commit_files: Some(50),
        granularity: "file".into(),
        ..Default::default()
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: AnalyzeSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.preset, "deep");
    assert_eq!(back.window, Some(128_000));
    assert_eq!(back.git, Some(true));
}

// ── TOML serialization ──────────────────────────────────────────────────

#[test]
fn toml_config_roundtrip() {
    let config = TomlConfig::default();
    let toml_str = toml::to_string(&config).unwrap();
    let back: TomlConfig = toml::from_str(&toml_str).unwrap();
    // Defaults should survive serialization
    assert!(back.scan.paths.is_none());
    assert!(back.analyze.preset.is_none());
}

#[test]
fn toml_config_parse_minimal() {
    let toml_str = r#"
[scan]
hidden = true

[analyze]
preset = "health"
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(config.scan.hidden, Some(true));
    assert_eq!(config.analyze.preset, Some("health".into()));
}

// ── Removing optional fields doesn't break deserialization ───────────────

#[test]
fn missing_optional_fields_use_defaults() {
    // A minimal JSON with only required defaults
    let json = r#"{"excluded":[],"config":"auto","hidden":false,"no_ignore":false,"no_ignore_parent":false,"no_ignore_dot":false,"no_ignore_vcs":false,"treat_doc_strings_as_comments":false}"#;
    let opts: ScanOptions = serde_json::from_str(json).unwrap();
    assert!(!opts.hidden);
    assert_eq!(opts.config, ConfigMode::Auto);
}

#[test]
fn empty_json_deserializes_to_defaults_for_scan_options() {
    // ScanOptions has #[serde(default)] on all fields
    let opts: ScanOptions = serde_json::from_str("{}").unwrap();
    assert!(opts.excluded.is_empty());
    assert!(!opts.hidden);
    assert_eq!(opts.config, ConfigMode::Auto);
}

#[test]
fn empty_json_deserializes_to_defaults_for_toml_config() {
    let config: TomlConfig = toml::from_str("").unwrap();
    assert!(config.scan.paths.is_none());
    assert!(config.analyze.preset.is_none());
    assert!(config.module.roots.is_none());
}

#[test]
fn partial_analyze_settings_deserializes() {
    let json = r#"{"preset":"risk"}"#;
    let s: AnalyzeSettings = serde_json::from_str(json).unwrap();
    assert_eq!(s.preset, "risk");
    // Other fields should get defaults
    assert!(s.window.is_none());
    assert!(s.git.is_none());
    assert_eq!(s.granularity, "module");
}

// ── Serde aliases work correctly ─────────────────────────────────────────

#[test]
fn config_mode_serde_variants() {
    let auto: ConfigMode = serde_json::from_str("\"auto\"").unwrap();
    assert_eq!(auto, ConfigMode::Auto);
    let none: ConfigMode = serde_json::from_str("\"none\"").unwrap();
    assert_eq!(none, ConfigMode::None);
}

#[test]
fn children_mode_serde_variants() {
    let collapse: ChildrenMode = serde_json::from_str("\"collapse\"").unwrap();
    assert_eq!(collapse, ChildrenMode::Collapse);
    let separate: ChildrenMode = serde_json::from_str("\"separate\"").unwrap();
    assert_eq!(separate, ChildrenMode::Separate);
}

#[test]
fn child_include_mode_serde_variants() {
    let sep: ChildIncludeMode = serde_json::from_str("\"separate\"").unwrap();
    assert_eq!(sep, ChildIncludeMode::Separate);
    let po: ChildIncludeMode = serde_json::from_str("\"parents-only\"").unwrap();
    assert_eq!(po, ChildIncludeMode::ParentsOnly);
}

#[test]
fn export_format_serde_variants() {
    for (s, expected) in [
        ("\"jsonl\"", ExportFormat::Jsonl),
        ("\"csv\"", ExportFormat::Csv),
        ("\"json\"", ExportFormat::Json),
        ("\"cyclonedx\"", ExportFormat::Cyclonedx),
    ] {
        let v: ExportFormat = serde_json::from_str(s).unwrap();
        assert_eq!(v, expected);
    }
}

#[test]
fn redact_mode_serde_variants() {
    for (s, expected) in [
        ("\"none\"", RedactMode::None),
        ("\"paths\"", RedactMode::Paths),
        ("\"all\"", RedactMode::All),
    ] {
        let v: RedactMode = serde_json::from_str(s).unwrap();
        assert_eq!(v, expected);
    }
}

// ── Property tests ───────────────────────────────────────────────────────

mod properties {
    use proptest::prelude::*;
    use tokmd_settings::ScanOptions;

    proptest! {
        #[test]
        fn scan_options_roundtrip(
            hidden in any::<bool>(),
            no_ignore in any::<bool>(),
        ) {
            let opts = ScanOptions {
                hidden,
                no_ignore,
                ..Default::default()
            };
            let json = serde_json::to_string(&opts).unwrap();
            let back: ScanOptions = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(back.hidden, hidden);
            prop_assert_eq!(back.no_ignore, no_ignore);
        }
    }
}
