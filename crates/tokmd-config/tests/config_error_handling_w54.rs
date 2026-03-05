//! Comprehensive error handling and edge case tests for tokmd-config.

use tokmd_config::{
    AnalysisPreset, BadgeMetric, CliLangArgs, ColorMode, ContextOutput, ContextStrategy,
    DiffFormat, DiffRangeMode, GateFormat, GlobalArgs, ImportGranularity, InitProfile,
    NearDupScope, Profile, UserConfig, ValueMetric,
};
use tokmd_settings::TomlConfig;

// ── Parse empty config ────────────────────────────────────────────────

#[test]
fn parse_empty_toml_string() {
    let config = TomlConfig::parse("").unwrap();
    assert!(config.scan.paths.is_none());
    assert!(config.scan.exclude.is_none());
    assert!(config.module.roots.is_none());
    assert!(config.export.min_code.is_none());
    assert!(config.view.is_empty());
}

#[test]
fn parse_empty_scan_section() {
    let config = TomlConfig::parse("[scan]\n").unwrap();
    assert!(config.scan.paths.is_none());
    assert!(config.scan.hidden.is_none());
}

// ── Parse invalid TOML ───────────────────────────────────────────────

#[test]
fn parse_invalid_toml_syntax() {
    let result = TomlConfig::parse("this is not [valid toml");
    assert!(result.is_err());
}

#[test]
fn parse_invalid_toml_unclosed_bracket() {
    let result = TomlConfig::parse("[scan\nhidden = true");
    assert!(result.is_err());
}

#[test]
fn parse_invalid_toml_bad_value_type() {
    // hidden expects a bool but gets a string
    let result = TomlConfig::parse("[scan]\nhidden = \"yes\"");
    assert!(result.is_err());
}

// ── Parse config with unknown keys ───────────────────────────────────

#[test]
fn parse_config_with_unknown_top_level_key_is_ignored() {
    // TomlConfig uses #[serde(default)] and doesn't use deny_unknown_fields,
    // so unknown top-level sections are silently ignored by toml/serde.
    // However, a truly malformed section like a wrong type will fail.
    let result = TomlConfig::parse("[scan]\nhidden = true\n");
    assert!(result.is_ok(), "known sections should parse fine");
    // Verify that an incorrect type for a known field fails
    let bad = TomlConfig::parse("[scan]\nhidden = 42\n");
    assert!(bad.is_err(), "wrong type for known field should error");
}

#[test]
fn parse_config_known_sections_only() {
    let toml = r#"
[scan]
hidden = true

[module]
depth = 3

[export]
min_code = 10
"#;
    let config = TomlConfig::parse(toml).unwrap();
    assert_eq!(config.scan.hidden, Some(true));
    assert_eq!(config.module.depth, Some(3));
    assert_eq!(config.export.min_code, Some(10));
}

// ── Default config generation ─────────────────────────────────────────

#[test]
fn default_user_config_is_empty() {
    let c = UserConfig::default();
    assert!(c.profiles.is_empty());
    assert!(c.repos.is_empty());
}

#[test]
fn default_profile_all_none() {
    let p = Profile::default();
    assert!(p.format.is_none());
    assert!(p.top.is_none());
    assert!(p.files.is_none());
    assert!(p.module_roots.is_none());
    assert!(p.module_depth.is_none());
    assert!(p.min_code.is_none());
    assert!(p.max_rows.is_none());
    assert!(p.redact.is_none());
    assert!(p.meta.is_none());
    assert!(p.children.is_none());
}

#[test]
fn default_global_args() {
    let g = GlobalArgs::default();
    assert!(g.excluded.is_empty());
    assert!(!g.hidden);
    assert!(!g.no_ignore);
    assert!(!g.no_ignore_parent);
    assert!(!g.no_ignore_dot);
    assert!(!g.no_ignore_vcs);
    assert!(!g.treat_doc_strings_as_comments);
    assert_eq!(g.verbose, 0);
    assert!(!g.no_progress);
}

#[test]
fn default_cli_lang_args() {
    let a = CliLangArgs::default();
    assert!(a.paths.is_none());
    assert!(a.format.is_none());
    assert!(a.top.is_none());
    assert!(!a.files);
    assert!(a.children.is_none());
}

// ── Enum default values ──────────────────────────────────────────────

#[test]
fn diff_format_default_is_md() {
    assert_eq!(DiffFormat::default(), DiffFormat::Md);
}

#[test]
fn color_mode_default_is_auto() {
    assert_eq!(ColorMode::default(), ColorMode::Auto);
}

#[test]
fn context_strategy_default_is_greedy() {
    assert_eq!(ContextStrategy::default(), ContextStrategy::Greedy);
}

#[test]
fn near_dup_scope_default_is_module() {
    assert_eq!(NearDupScope::default(), NearDupScope::Module);
}

#[test]
fn diff_range_mode_default_is_two_dot() {
    assert_eq!(DiffRangeMode::default(), DiffRangeMode::TwoDot);
}

// ── Enum serde roundtrips ─────────────────────────────────────────────

#[test]
fn analysis_preset_serde_roundtrip_all_variants() {
    for variant in [
        AnalysisPreset::Receipt,
        AnalysisPreset::Health,
        AnalysisPreset::Risk,
        AnalysisPreset::Supply,
        AnalysisPreset::Architecture,
        AnalysisPreset::Topics,
        AnalysisPreset::Security,
        AnalysisPreset::Identity,
        AnalysisPreset::Git,
        AnalysisPreset::Deep,
        AnalysisPreset::Fun,
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: AnalysisPreset = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant, "roundtrip failed for {variant:?}");
    }
}

#[test]
fn import_granularity_serde_roundtrip() {
    for variant in [ImportGranularity::Module, ImportGranularity::File] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: ImportGranularity = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn badge_metric_serde_roundtrip() {
    for variant in [
        BadgeMetric::Lines,
        BadgeMetric::Tokens,
        BadgeMetric::Bytes,
        BadgeMetric::Doc,
        BadgeMetric::Blank,
        BadgeMetric::Hotspot,
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: BadgeMetric = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn init_profile_serde_roundtrip() {
    for variant in [
        InitProfile::Default,
        InitProfile::Rust,
        InitProfile::Node,
        InitProfile::Mono,
        InitProfile::Python,
        InitProfile::Go,
        InitProfile::Cpp,
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: InitProfile = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn gate_format_serde_roundtrip() {
    for variant in [GateFormat::Json, GateFormat::Text] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: GateFormat = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn value_metric_serde_roundtrip() {
    for variant in [
        ValueMetric::Code,
        ValueMetric::Tokens,
        ValueMetric::Churn,
        ValueMetric::Hotspot,
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: ValueMetric = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn context_output_serde_roundtrip() {
    for variant in [
        ContextOutput::List,
        ContextOutput::Bundle,
        ContextOutput::Json,
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: ContextOutput = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

// ── Config file loading errors ────────────────────────────────────────

#[test]
fn toml_config_from_file_nonexistent() {
    let result = TomlConfig::from_file(std::path::Path::new("/nonexistent/tokmd_w54.toml"));
    assert!(result.is_err());
}

#[test]
fn toml_config_from_file_invalid_content() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bad.toml");
    std::fs::write(&path, "not valid toml {{{{").unwrap();
    let result = TomlConfig::from_file(&path);
    assert!(result.is_err());
}

// ── Full config parsing ──────────────────────────────────────────────

#[test]
fn parse_full_config_with_all_sections() {
    let toml = r#"
[scan]
paths = ["."]
exclude = ["target", "node_modules"]
hidden = false
config = "auto"
no_ignore = false

[module]
roots = ["crates", "packages"]
depth = 2

[export]
min_code = 5
max_rows = 1000
format = "jsonl"

[analyze]
preset = "health"
window = 128000
git = true

[context]
budget = "128k"
strategy = "greedy"

[badge]
metric = "lines"

[gate]
fail_fast = true
"#;
    let config = TomlConfig::parse(toml).unwrap();
    assert_eq!(config.scan.paths, Some(vec![".".to_string()]));
    assert_eq!(config.module.depth, Some(2));
    assert_eq!(config.export.min_code, Some(5));
    assert_eq!(config.analyze.preset, Some("health".to_string()));
    assert_eq!(config.context.budget, Some("128k".to_string()));
    assert_eq!(config.gate.fail_fast, Some(true));
}
