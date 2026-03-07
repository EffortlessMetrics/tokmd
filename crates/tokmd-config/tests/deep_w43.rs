//! Wave 43 deep tests for `tokmd-config`.
//!
//! Covers: GlobalArgs defaults, command-specific settings, config file
//! loading from tokmd.toml, config merge behavior (CLI overrides file),
//! enum variant serde, UserConfig/Profile, and conversions.

use std::io::Write;

use tempfile::NamedTempFile;
use tokmd_config::{
    AnalysisPreset, BadgeMetric, CliLangArgs, CockpitFormat, ColorMode, ContextOutput,
    ContextStrategy, DiffFormat, DiffRangeMode, GateFormat, GlobalArgs, HandoffPreset,
    ImportGranularity, InitProfile, NearDupScope, Profile, RedactMode, SensorFormat, Shell,
    TomlConfig, UserConfig, ValueMetric, ViewProfile,
};

// =============================================================================
// 1. GlobalArgs defaults
// =============================================================================

#[test]
fn global_args_default_excluded_empty() {
    let g = GlobalArgs::default();
    assert!(g.excluded.is_empty());
}

#[test]
fn global_args_default_config_is_auto() {
    let g = GlobalArgs::default();
    assert_eq!(g.config, tokmd_config::ConfigMode::Auto);
}

#[test]
fn global_args_default_booleans_false() {
    let g = GlobalArgs::default();
    assert!(!g.hidden);
    assert!(!g.no_ignore);
    assert!(!g.no_ignore_parent);
    assert!(!g.no_ignore_dot);
    assert!(!g.no_ignore_vcs);
    assert!(!g.treat_doc_strings_as_comments);
    assert!(!g.no_progress);
}

#[test]
fn global_args_default_verbose_zero() {
    let g = GlobalArgs::default();
    assert_eq!(g.verbose, 0);
}

// =============================================================================
// 2. Command-specific settings defaults
// =============================================================================

#[test]
fn cli_lang_args_default_all_none() {
    let a = CliLangArgs::default();
    assert!(a.paths.is_none());
    assert!(a.format.is_none());
    assert!(a.top.is_none());
    assert!(!a.files);
    assert!(a.children.is_none());
}

#[test]
fn diff_format_default_md() {
    assert_eq!(DiffFormat::default(), DiffFormat::Md);
}

#[test]
fn color_mode_default_auto() {
    assert_eq!(ColorMode::default(), ColorMode::Auto);
}

#[test]
fn gate_format_default_text() {
    assert_eq!(GateFormat::default(), GateFormat::Text);
}

#[test]
fn cockpit_format_default_json() {
    assert_eq!(CockpitFormat::default(), CockpitFormat::Json);
}

#[test]
fn handoff_preset_default_risk() {
    assert_eq!(HandoffPreset::default(), HandoffPreset::Risk);
}

#[test]
fn context_strategy_default_greedy() {
    assert_eq!(ContextStrategy::default(), ContextStrategy::Greedy);
}

#[test]
fn context_output_default_list() {
    assert_eq!(ContextOutput::default(), ContextOutput::List);
}

#[test]
fn value_metric_default_code() {
    assert_eq!(ValueMetric::default(), ValueMetric::Code);
}

// =============================================================================
// 3. Enum serde roundtrips (kebab-case)
// =============================================================================

#[test]
fn analysis_preset_all_variants_roundtrip() {
    let variants = [
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
        AnalysisPreset::Estimate,
    ];
    for v in variants {
        let json = serde_json::to_string(&v).unwrap();
        let back: AnalysisPreset = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
}

#[test]
fn shell_enum_serde_roundtrip() {
    for v in [
        Shell::Bash,
        Shell::Elvish,
        Shell::Fish,
        Shell::Powershell,
        Shell::Zsh,
    ] {
        let json = serde_json::to_string(&v).unwrap();
        let back: Shell = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
}

#[test]
fn init_profile_serde_roundtrip() {
    for v in [
        InitProfile::Default,
        InitProfile::Rust,
        InitProfile::Node,
        InitProfile::Mono,
        InitProfile::Python,
        InitProfile::Go,
        InitProfile::Cpp,
    ] {
        let json = serde_json::to_string(&v).unwrap();
        let back: InitProfile = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
}

#[test]
fn import_granularity_serde_roundtrip() {
    for v in [ImportGranularity::Module, ImportGranularity::File] {
        let json = serde_json::to_string(&v).unwrap();
        let back: ImportGranularity = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
}

#[test]
fn badge_metric_serde_roundtrip() {
    for v in [
        BadgeMetric::Lines,
        BadgeMetric::Tokens,
        BadgeMetric::Bytes,
        BadgeMetric::Doc,
        BadgeMetric::Blank,
        BadgeMetric::Hotspot,
    ] {
        let json = serde_json::to_string(&v).unwrap();
        let back: BadgeMetric = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
}

#[test]
fn near_dup_scope_serde_roundtrip() {
    for v in [
        NearDupScope::Module,
        NearDupScope::Lang,
        NearDupScope::Global,
    ] {
        let json = serde_json::to_string(&v).unwrap();
        let back: NearDupScope = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
}

#[test]
fn diff_range_mode_serde_roundtrip() {
    for v in [DiffRangeMode::TwoDot, DiffRangeMode::ThreeDot] {
        let json = serde_json::to_string(&v).unwrap();
        let back: DiffRangeMode = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
}

#[test]
fn sensor_format_serde_roundtrip() {
    for v in [SensorFormat::Json, SensorFormat::Md] {
        let json = serde_json::to_string(&v).unwrap();
        let back: SensorFormat = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
}

// =============================================================================
// 4. UserConfig and Profile
// =============================================================================

#[test]
fn user_config_default_empty() {
    let c = UserConfig::default();
    assert!(c.profiles.is_empty());
    assert!(c.repos.is_empty());
}

#[test]
fn profile_default_all_none() {
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
fn user_config_serde_roundtrip_multiple_profiles() {
    let mut c = UserConfig::default();
    c.profiles.insert(
        "ci".into(),
        Profile {
            format: Some("json".into()),
            top: Some(0),
            ..Default::default()
        },
    );
    c.profiles.insert(
        "llm_safe".into(),
        Profile {
            format: Some("json".into()),
            redact: Some(RedactMode::All),
            top: Some(10),
            ..Default::default()
        },
    );
    c.repos.insert("org/repo-a".into(), "ci".into());
    c.repos.insert("org/repo-b".into(), "llm_safe".into());

    let json = serde_json::to_string(&c).unwrap();
    let back: UserConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(back.profiles.len(), 2);
    assert_eq!(back.repos.len(), 2);
    assert_eq!(back.profiles["ci"].top, Some(0));
    assert_eq!(back.profiles["llm_safe"].redact, Some(RedactMode::All));
}

#[test]
fn user_config_profiles_btreemap_sorted() {
    let mut c = UserConfig::default();
    c.profiles.insert("z_last".into(), Profile::default());
    c.profiles.insert("a_first".into(), Profile::default());
    c.profiles.insert("m_middle".into(), Profile::default());

    let keys: Vec<&String> = c.profiles.keys().collect();
    assert_eq!(keys, vec!["a_first", "m_middle", "z_last"]);
}

#[test]
fn user_config_repos_btreemap_sorted() {
    let mut c = UserConfig::default();
    c.repos.insert("z/repo".into(), "profile".into());
    c.repos.insert("a/repo".into(), "profile".into());

    let keys: Vec<&String> = c.repos.keys().collect();
    assert_eq!(keys, vec!["a/repo", "z/repo"]);
}

// =============================================================================
// 5. Config file loading from tokmd.toml
// =============================================================================

#[test]
fn toml_config_empty_string_all_defaults() {
    let cfg = TomlConfig::parse("").unwrap();
    assert!(cfg.scan.paths.is_none());
    assert!(cfg.scan.exclude.is_none());
    assert!(cfg.module.roots.is_none());
    assert!(cfg.module.depth.is_none());
    assert!(cfg.export.min_code.is_none());
    assert!(cfg.analyze.preset.is_none());
    assert!(cfg.context.budget.is_none());
    assert!(cfg.badge.metric.is_none());
    assert!(cfg.gate.policy.is_none());
    assert!(cfg.view.is_empty());
}

#[test]
fn toml_config_scan_section_parses() {
    let toml_str = r#"
[scan]
paths = ["src", "lib"]
exclude = ["target", "*.bak"]
hidden = true
config = "none"
no_ignore = true
no_ignore_parent = true
no_ignore_dot = true
no_ignore_vcs = true
doc_comments = true
"#;
    let cfg = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(
        cfg.scan.paths,
        Some(vec!["src".to_string(), "lib".to_string()])
    );
    assert_eq!(cfg.scan.hidden, Some(true));
    assert_eq!(cfg.scan.config, Some("none".to_string()));
    assert_eq!(cfg.scan.no_ignore, Some(true));
    assert_eq!(cfg.scan.doc_comments, Some(true));
}

#[test]
fn toml_config_module_section_parses() {
    let toml_str = r#"
[module]
roots = ["crates", "packages", "libs"]
depth = 3
children = "collapse"
"#;
    let cfg = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(cfg.module.depth, Some(3));
    assert_eq!(
        cfg.module.roots,
        Some(vec![
            "crates".to_string(),
            "packages".to_string(),
            "libs".to_string()
        ])
    );
    assert_eq!(cfg.module.children, Some("collapse".to_string()));
}

#[test]
fn toml_config_analyze_section_all_fields() {
    let toml_str = r#"
[analyze]
preset = "deep"
window = 200000
format = "json"
git = false
max_files = 10000
max_bytes = 100000000
max_file_bytes = 2000000
max_commits = 1000
max_commit_files = 200
granularity = "file"
"#;
    let cfg = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(cfg.analyze.preset, Some("deep".to_string()));
    assert_eq!(cfg.analyze.window, Some(200_000));
    assert_eq!(cfg.analyze.git, Some(false));
    assert_eq!(cfg.analyze.max_files, Some(10_000));
    assert_eq!(cfg.analyze.max_bytes, Some(100_000_000));
    assert_eq!(cfg.analyze.max_file_bytes, Some(2_000_000));
    assert_eq!(cfg.analyze.max_commits, Some(1000));
    assert_eq!(cfg.analyze.granularity, Some("file".to_string()));
}

#[test]
fn toml_config_view_profiles_multiple() {
    let toml_str = r#"
[view.llm]
format = "json"
top = 10
redact = "all"
files = true

[view.ci]
format = "json"
preset = "health"

[view.minimal]
top = 5
"#;
    let cfg = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(cfg.view.len(), 3);

    let llm = cfg.view.get("llm").unwrap();
    assert_eq!(llm.format.as_deref(), Some("json"));
    assert_eq!(llm.top, Some(10));
    assert_eq!(llm.redact.as_deref(), Some("all"));
    assert_eq!(llm.files, Some(true));

    let ci = cfg.view.get("ci").unwrap();
    assert_eq!(ci.preset.as_deref(), Some("health"));

    let minimal = cfg.view.get("minimal").unwrap();
    assert_eq!(minimal.top, Some(5));
    assert!(minimal.format.is_none());
}

#[test]
fn toml_config_from_file_reads() {
    let content = r#"
[scan]
hidden = true

[module]
depth = 4

[view.test]
format = "tsv"
"#;
    let mut tmp = NamedTempFile::new().unwrap();
    tmp.write_all(content.as_bytes()).unwrap();

    let cfg = TomlConfig::from_file(tmp.path()).unwrap();
    assert_eq!(cfg.scan.hidden, Some(true));
    assert_eq!(cfg.module.depth, Some(4));
    assert_eq!(cfg.view.get("test").unwrap().format.as_deref(), Some("tsv"));
}

#[test]
fn toml_config_from_missing_file_is_error() {
    let result = TomlConfig::from_file(std::path::Path::new("nonexistent_tokmd_w43.toml"));
    assert!(result.is_err());
}

#[test]
fn toml_config_invalid_toml_is_error() {
    let result = TomlConfig::parse("[[[invalid toml syntax");
    assert!(result.is_err());
}

// =============================================================================
// 6. GlobalArgs → ScanOptions conversion
// =============================================================================

#[test]
fn global_args_to_scan_options_ref_conversion() {
    let g = GlobalArgs {
        excluded: vec!["target".into(), "vendor".into()],
        config: tokmd_config::ConfigMode::None,
        hidden: true,
        no_ignore: true,
        no_ignore_parent: false,
        no_ignore_dot: true,
        no_ignore_vcs: false,
        treat_doc_strings_as_comments: true,
        verbose: 2,
        no_progress: true,
    };
    let opts: tokmd_settings::ScanOptions = (&g).into();
    assert_eq!(opts.excluded, vec!["target", "vendor"]);
    assert!(opts.hidden);
    assert!(opts.no_ignore);
    assert!(!opts.no_ignore_parent);
    assert!(opts.no_ignore_dot);
    assert!(!opts.no_ignore_vcs);
    assert!(opts.treat_doc_strings_as_comments);
}

#[test]
fn global_args_to_scan_options_owned_conversion() {
    let g = GlobalArgs {
        excluded: vec!["dist".into()],
        hidden: false,
        ..GlobalArgs::default()
    };
    let opts: tokmd_settings::ScanOptions = g.into();
    assert_eq!(opts.excluded, vec!["dist"]);
    assert!(!opts.hidden);
}

#[test]
fn global_args_conversion_drops_verbose_and_no_progress() {
    let g = GlobalArgs {
        verbose: 3,
        no_progress: true,
        ..GlobalArgs::default()
    };
    // ScanOptions has no verbose/no_progress fields — just verify conversion works
    let opts: tokmd_settings::ScanOptions = (&g).into();
    assert!(opts.excluded.is_empty());
    assert!(!opts.hidden);
}

// =============================================================================
// 7. Gate config with inline rules
// =============================================================================

#[test]
fn toml_gate_inline_rules_parse() {
    let toml_str = r#"
[[gate.rules]]
name = "max_lines"
pointer = "/summary/total_code"
op = "<="
value = 100000

[[gate.rules]]
name = "language_check"
pointer = "/summary/dominant_language"
op = "in"
values = ["Rust", "Go", "Python"]
negate = true
level = "warn"
message = "Unexpected dominant language"
"#;
    let cfg = TomlConfig::parse(toml_str).unwrap();
    let rules = cfg.gate.rules.unwrap();
    assert_eq!(rules.len(), 2);
    assert_eq!(rules[0].name, "max_lines");
    assert_eq!(rules[0].pointer, "/summary/total_code");
    assert_eq!(rules[0].op, "<=");
    assert!(!rules[0].negate);

    assert_eq!(rules[1].name, "language_check");
    assert!(rules[1].negate);
    assert_eq!(rules[1].level.as_deref(), Some("warn"));
    assert!(rules[1].values.is_some());
}

#[test]
fn toml_gate_ratchet_rules_parse() {
    let toml_str = r#"
[[gate.ratchet]]
pointer = "/complexity/max_cyclomatic"
max_increase_pct = 10.0

[[gate.ratchet]]
pointer = "/todo_density"
max_value = 0.05
level = "error"
description = "TODO density ceiling"
"#;
    let cfg = TomlConfig::parse(toml_str).unwrap();
    let ratchet = cfg.gate.ratchet.unwrap();
    assert_eq!(ratchet.len(), 2);
    assert_eq!(ratchet[0].max_increase_pct, Some(10.0));
    assert!(ratchet[0].max_value.is_none());
    assert_eq!(ratchet[1].max_value, Some(0.05));
    assert_eq!(
        ratchet[1].description.as_deref(),
        Some("TODO density ceiling")
    );
}

// =============================================================================
// 8. ViewProfile completeness
// =============================================================================

#[test]
fn view_profile_default_all_none() {
    let vp = ViewProfile::default();
    assert!(vp.format.is_none());
    assert!(vp.top.is_none());
    assert!(vp.files.is_none());
    assert!(vp.module_roots.is_none());
    assert!(vp.module_depth.is_none());
    assert!(vp.min_code.is_none());
    assert!(vp.max_rows.is_none());
    assert!(vp.redact.is_none());
    assert!(vp.meta.is_none());
    assert!(vp.children.is_none());
    assert!(vp.preset.is_none());
    assert!(vp.window.is_none());
    assert!(vp.budget.is_none());
    assert!(vp.strategy.is_none());
    assert!(vp.rank_by.is_none());
    assert!(vp.output.is_none());
    assert!(vp.compress.is_none());
    assert!(vp.metric.is_none());
}

#[test]
fn view_profile_full_serde_roundtrip() {
    let vp = ViewProfile {
        format: Some("json".into()),
        top: Some(10),
        files: Some(true),
        module_roots: Some(vec!["crates".into()]),
        module_depth: Some(3),
        min_code: Some(5),
        max_rows: Some(500),
        redact: Some("paths".into()),
        meta: Some(false),
        children: Some("collapse".into()),
        preset: Some("risk".into()),
        window: Some(128_000),
        budget: Some("256k".into()),
        strategy: Some("spread".into()),
        rank_by: Some("hotspot".into()),
        output: Some("bundle".into()),
        compress: Some(true),
        metric: Some("tokens".into()),
    };
    let json = serde_json::to_string(&vp).unwrap();
    let back: ViewProfile = serde_json::from_str(&json).unwrap();
    assert_eq!(back.format.as_deref(), Some("json"));
    assert_eq!(back.top, Some(10));
    assert_eq!(back.files, Some(true));
    assert_eq!(back.module_depth, Some(3));
    assert_eq!(back.min_code, Some(5));
    assert_eq!(back.compress, Some(true));
    assert_eq!(back.metric.as_deref(), Some("tokens"));
}

// =============================================================================
// 9. kebab-case serialization checks
// =============================================================================

#[test]
fn handoff_preset_uses_kebab_case() {
    assert_eq!(
        serde_json::to_string(&HandoffPreset::Minimal).unwrap(),
        "\"minimal\""
    );
    assert_eq!(
        serde_json::to_string(&HandoffPreset::Standard).unwrap(),
        "\"standard\""
    );
    assert_eq!(
        serde_json::to_string(&HandoffPreset::Risk).unwrap(),
        "\"risk\""
    );
    assert_eq!(
        serde_json::to_string(&HandoffPreset::Deep).unwrap(),
        "\"deep\""
    );
}

#[test]
fn diff_range_mode_uses_kebab_case() {
    assert_eq!(
        serde_json::to_string(&DiffRangeMode::TwoDot).unwrap(),
        "\"two-dot\""
    );
    assert_eq!(
        serde_json::to_string(&DiffRangeMode::ThreeDot).unwrap(),
        "\"three-dot\""
    );
}
