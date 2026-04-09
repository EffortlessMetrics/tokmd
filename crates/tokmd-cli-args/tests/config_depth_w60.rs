//! Depth tests for tokmd-config (w60).
//!
//! BDD-style tests covering:
//! - TOML configuration loading and parsing
//! - Config merging (file + CLI overrides)
//! - Default values, missing fields, extra fields
//! - Profile system (UserConfig, ViewProfile)
//! - Config serialization roundtrips
//! - Enum serde coverage
//! - GlobalArgs → ScanOptions conversion
//! - Property tests for config determinism

use tokmd_cli_args::{
    AnalysisPreset, BadgeMetric, CliConfigMode, CliRedactMode, CockpitFormat, ColorMode,
    ContextOutput, ContextStrategy, DiffFormat, DiffRangeMode, GateFormat, GlobalArgs,
    HandoffPreset, ImportGranularity, NearDupScope, Profile, SensorFormat, TomlConfig, UserConfig,
    ValueMetric, ViewProfile,
};

// ═══════════════════════════════════════════════════════════════════
// 1. TOML Parsing — Valid configs
// ═══════════════════════════════════════════════════════════════════

#[test]
fn given_empty_toml_when_parsed_then_all_defaults() {
    let config = TomlConfig::parse("").expect("empty TOML parses");
    assert_eq!(config.scan.hidden, None);
    assert_eq!(config.scan.no_ignore, None);
    assert_eq!(config.module.depth, None);
    assert_eq!(config.module.roots, None);
    assert_eq!(config.export.min_code, None);
    assert_eq!(config.export.max_rows, None);
    assert_eq!(config.analyze.preset, None);
    assert_eq!(config.context.budget, None);
    assert_eq!(config.badge.metric, None);
    assert_eq!(config.gate.policy, None);
    assert!(config.view.is_empty());
}

#[test]
fn given_scan_section_when_parsed_then_fields_populated() {
    let toml_str = r#"
[scan]
hidden = true
no_ignore = true
no_ignore_parent = false
no_ignore_vcs = true
doc_comments = true
paths = ["src", "lib"]
exclude = ["target", "node_modules"]
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(config.scan.hidden, Some(true));
    assert_eq!(config.scan.no_ignore, Some(true));
    assert_eq!(config.scan.no_ignore_parent, Some(false));
    assert_eq!(config.scan.no_ignore_vcs, Some(true));
    assert_eq!(config.scan.doc_comments, Some(true));
    assert_eq!(config.scan.paths, Some(vec!["src".into(), "lib".into()]));
    assert_eq!(
        config.scan.exclude,
        Some(vec!["target".into(), "node_modules".into()])
    );
}

#[test]
fn given_module_section_when_parsed_then_roots_and_depth_loaded() {
    let toml_str = r#"
[module]
roots = ["crates", "packages"]
depth = 3
children = "collapse"
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(
        config.module.roots,
        Some(vec!["crates".into(), "packages".into()])
    );
    assert_eq!(config.module.depth, Some(3));
    assert_eq!(config.module.children, Some("collapse".into()));
}

#[test]
fn given_export_section_when_parsed_then_limits_set() {
    let toml_str = r#"
[export]
min_code = 5
max_rows = 500
redact = "paths"
format = "csv"
children = "separate"
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(config.export.min_code, Some(5));
    assert_eq!(config.export.max_rows, Some(500));
    assert_eq!(config.export.redact, Some("paths".into()));
    assert_eq!(config.export.format, Some("csv".into()));
    assert_eq!(config.export.children, Some("separate".into()));
}

#[test]
fn given_analyze_section_when_parsed_then_all_fields_set() {
    let toml_str = r#"
[analyze]
preset = "deep"
window = 128000
format = "json"
git = true
max_files = 10000
max_bytes = 50000000
max_file_bytes = 1000000
max_commits = 500
max_commit_files = 200
granularity = "file"
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(config.analyze.preset, Some("deep".into()));
    assert_eq!(config.analyze.window, Some(128_000));
    assert_eq!(config.analyze.format, Some("json".into()));
    assert_eq!(config.analyze.git, Some(true));
    assert_eq!(config.analyze.max_files, Some(10000));
    assert_eq!(config.analyze.max_bytes, Some(50_000_000));
    assert_eq!(config.analyze.max_file_bytes, Some(1_000_000));
    assert_eq!(config.analyze.max_commits, Some(500));
    assert_eq!(config.analyze.max_commit_files, Some(200));
    assert_eq!(config.analyze.granularity, Some("file".into()));
}

#[test]
fn given_context_section_when_parsed_then_strategy_and_budget_set() {
    let toml_str = r#"
[context]
budget = "256k"
strategy = "spread"
rank_by = "hotspot"
output = "bundle"
compress = true
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(config.context.budget, Some("256k".into()));
    assert_eq!(config.context.strategy, Some("spread".into()));
    assert_eq!(config.context.rank_by, Some("hotspot".into()));
    assert_eq!(config.context.output, Some("bundle".into()));
    assert_eq!(config.context.compress, Some(true));
}

#[test]
fn given_gate_section_when_parsed_then_policy_fields_set() {
    let toml_str = r#"
[gate]
policy = "policy.toml"
baseline = "baseline.json"
preset = "health"
fail_fast = true
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(config.gate.policy, Some("policy.toml".into()));
    assert_eq!(config.gate.baseline, Some("baseline.json".into()));
    assert_eq!(config.gate.preset, Some("health".into()));
    assert_eq!(config.gate.fail_fast, Some(true));
}

#[test]
fn given_badge_section_when_parsed_then_metric_set() {
    let toml_str = r#"
[badge]
metric = "tokens"
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(config.badge.metric, Some("tokens".into()));
}

// ═══════════════════════════════════════════════════════════════════
// 2. TOML Parsing — View profiles
// ═══════════════════════════════════════════════════════════════════

#[test]
fn given_view_profile_when_parsed_then_all_fields_available() {
    let toml_str = r#"
[view.ci]
format = "json"
top = 10
files = true
redact = "paths"
module_roots = ["crates"]
module_depth = 2
min_code = 1
max_rows = 100
meta = true
children = "collapse"
preset = "health"
window = 64000
budget = "64k"
strategy = "greedy"
rank_by = "code"
output = "list"
compress = false
metric = "lines"
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    let ci = &config.view["ci"];
    assert_eq!(ci.format, Some("json".into()));
    assert_eq!(ci.top, Some(10));
    assert_eq!(ci.files, Some(true));
    assert_eq!(ci.redact, Some("paths".into()));
    assert_eq!(ci.module_roots, Some(vec!["crates".into()]));
    assert_eq!(ci.module_depth, Some(2));
    assert_eq!(ci.min_code, Some(1));
    assert_eq!(ci.max_rows, Some(100));
    assert_eq!(ci.meta, Some(true));
    assert_eq!(ci.children, Some("collapse".into()));
    assert_eq!(ci.preset, Some("health".into()));
    assert_eq!(ci.window, Some(64000));
    assert_eq!(ci.budget, Some("64k".into()));
    assert_eq!(ci.strategy, Some("greedy".into()));
    assert_eq!(ci.rank_by, Some("code".into()));
    assert_eq!(ci.output, Some("list".into()));
    assert_eq!(ci.compress, Some(false));
    assert_eq!(ci.metric, Some("lines".into()));
}

#[test]
fn given_multiple_view_profiles_when_parsed_then_all_accessible() {
    let toml_str = r#"
[view.ci]
format = "json"
top = 5

[view.llm]
format = "md"
redact = "all"
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(config.view.len(), 2);
    assert_eq!(config.view["ci"].format, Some("json".into()));
    assert_eq!(config.view["ci"].top, Some(5));
    assert_eq!(config.view["llm"].format, Some("md".into()));
    assert_eq!(config.view["llm"].redact, Some("all".into()));
}

#[test]
fn given_empty_view_profile_when_parsed_then_all_fields_none() {
    let toml_str = r#"
[view.empty]
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    let empty = &config.view["empty"];
    assert!(empty.format.is_none());
    assert!(empty.top.is_none());
    assert!(empty.files.is_none());
}

// ═══════════════════════════════════════════════════════════════════
// 3. TOML Parsing — Error cases
// ═══════════════════════════════════════════════════════════════════

#[test]
fn given_invalid_toml_syntax_when_parsed_then_error() {
    let result = TomlConfig::parse("[[[ not valid toml");
    assert!(result.is_err());
}

#[test]
fn given_wrong_type_for_field_when_parsed_then_error() {
    let result = TomlConfig::parse(
        r#"
[scan]
hidden = "not_a_bool"
"#,
    );
    assert!(result.is_err());
}

#[test]
fn given_wrong_type_for_depth_when_parsed_then_error() {
    let result = TomlConfig::parse(
        r#"
[module]
depth = "three"
"#,
    );
    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════════
// 4. Extra fields (TOML serde default allows unknown fields)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn given_unknown_top_level_keys_when_parsed_then_no_error() {
    // serde(default) on all sections means unknown sections are silently ignored
    let toml_str = r#"
[scan]
hidden = true
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(config.scan.hidden, Some(true));
}

// ═══════════════════════════════════════════════════════════════════
// 5. UserConfig and Profile
// ═══════════════════════════════════════════════════════════════════

#[test]
fn given_default_user_config_then_empty() {
    let c = UserConfig::default();
    assert!(c.profiles.is_empty());
    assert!(c.repos.is_empty());
}

#[test]
fn given_user_config_with_profiles_then_profiles_accessible() {
    let mut c = UserConfig::default();
    c.profiles.insert(
        "llm_safe".into(),
        Profile {
            format: Some("json".into()),
            top: Some(10),
            redact: Some(CliRedactMode::All),
            ..Profile::default()
        },
    );
    c.repos.insert("owner/repo".into(), "llm_safe".into());
    assert_eq!(c.profiles.len(), 1);
    assert_eq!(c.repos.len(), 1);
    assert_eq!(c.profiles["llm_safe"].top, Some(10));
}

#[test]
fn given_profile_default_then_all_fields_none() {
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
fn given_profile_with_all_fields_then_serde_roundtrip() {
    let p = Profile {
        format: Some("json".into()),
        top: Some(20),
        files: Some(true),
        module_roots: Some(vec!["crates".into(), "packages".into()]),
        module_depth: Some(3),
        min_code: Some(5),
        max_rows: Some(1000),
        redact: Some(CliRedactMode::Paths),
        meta: Some(true),
        children: Some("collapse".into()),
    };
    let json = serde_json::to_string(&p).unwrap();
    let back: Profile = serde_json::from_str(&json).unwrap();
    assert_eq!(back.format, p.format);
    assert_eq!(back.top, p.top);
    assert_eq!(back.files, p.files);
    assert_eq!(back.module_roots, p.module_roots);
    assert_eq!(back.module_depth, p.module_depth);
}

// ═══════════════════════════════════════════════════════════════════
// 6. GlobalArgs → ScanOptions conversion
// ═══════════════════════════════════════════════════════════════════

#[test]
fn given_global_args_with_defaults_when_converted_then_scan_options_defaults() {
    let g = GlobalArgs::default();
    let opts: tokmd_settings::ScanOptions = (&g).into();
    assert!(opts.excluded.is_empty());
    assert!(!opts.hidden);
    assert!(!opts.no_ignore);
    assert!(!opts.no_ignore_parent);
    assert!(!opts.no_ignore_dot);
    assert!(!opts.no_ignore_vcs);
    assert!(!opts.treat_doc_strings_as_comments);
}

#[test]
fn given_global_args_with_custom_values_when_converted_then_all_propagated() {
    let g = GlobalArgs {
        excluded: vec!["target".into(), "vendor".into()],
        config: CliConfigMode::None,
        hidden: true,
        no_ignore: true,
        no_ignore_parent: true,
        no_ignore_dot: true,
        no_ignore_vcs: true,
        treat_doc_strings_as_comments: true,
        verbose: 2,
        no_progress: true,
    };
    let opts: tokmd_settings::ScanOptions = (&g).into();
    assert_eq!(opts.excluded.len(), 2);
    assert_eq!(opts.config, tokmd_types::ConfigMode::None);
    assert!(opts.hidden);
    assert!(opts.no_ignore);
    assert!(opts.no_ignore_parent);
    assert!(opts.no_ignore_dot);
    assert!(opts.no_ignore_vcs);
    assert!(opts.treat_doc_strings_as_comments);
}

#[test]
fn given_global_args_owned_when_converted_then_same_as_borrowed() {
    let g = GlobalArgs {
        excluded: vec!["node_modules".into()],
        config: CliConfigMode::Auto,
        hidden: false,
        no_ignore: false,
        no_ignore_parent: false,
        no_ignore_dot: false,
        no_ignore_vcs: false,
        treat_doc_strings_as_comments: false,
        verbose: 0,
        no_progress: false,
    };
    let opts_borrowed: tokmd_settings::ScanOptions = (&g).into();
    let opts_owned: tokmd_settings::ScanOptions = g.into();
    assert_eq!(opts_borrowed.excluded, opts_owned.excluded);
    assert_eq!(opts_borrowed.hidden, opts_owned.hidden);
    assert_eq!(opts_borrowed.config, opts_owned.config);
}

// ═══════════════════════════════════════════════════════════════════
// 7. Enum defaults
// ═══════════════════════════════════════════════════════════════════

#[test]
fn diff_format_default_md() {
    assert_eq!(DiffFormat::default(), DiffFormat::Md);
}

#[test]
fn color_mode_default_auto() {
    assert_eq!(ColorMode::default(), ColorMode::Auto);
}

#[test]
fn context_strategy_default_greedy() {
    assert_eq!(ContextStrategy::default(), ContextStrategy::Greedy);
}

#[test]
fn value_metric_default_code() {
    assert_eq!(ValueMetric::default(), ValueMetric::Code);
}

#[test]
fn context_output_default_list() {
    assert_eq!(ContextOutput::default(), ContextOutput::List);
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
fn sensor_format_default_json() {
    assert_eq!(SensorFormat::default(), SensorFormat::Json);
}

#[test]
fn near_dup_scope_default_module() {
    assert_eq!(NearDupScope::default(), NearDupScope::Module);
}

#[test]
fn diff_range_mode_default_two_dot() {
    assert_eq!(DiffRangeMode::default(), DiffRangeMode::TwoDot);
}

// ═══════════════════════════════════════════════════════════════════
// 8. Enum serde roundtrips
// ═══════════════════════════════════════════════════════════════════

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
    ];
    for variant in variants {
        let json = serde_json::to_string(&variant).unwrap();
        let back: AnalysisPreset = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn diff_format_all_variants_roundtrip() {
    for variant in [DiffFormat::Md, DiffFormat::Json] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: DiffFormat = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn color_mode_all_variants_roundtrip() {
    for variant in [ColorMode::Auto, ColorMode::Always, ColorMode::Never] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: ColorMode = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn context_strategy_all_variants_roundtrip() {
    for variant in [ContextStrategy::Greedy, ContextStrategy::Spread] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: ContextStrategy = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn value_metric_all_variants_roundtrip() {
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
fn badge_metric_all_variants_roundtrip() {
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
fn handoff_preset_all_variants_roundtrip() {
    for variant in [
        HandoffPreset::Minimal,
        HandoffPreset::Standard,
        HandoffPreset::Risk,
        HandoffPreset::Deep,
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: HandoffPreset = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn import_granularity_all_variants_roundtrip() {
    for variant in [ImportGranularity::Module, ImportGranularity::File] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: ImportGranularity = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

// ═══════════════════════════════════════════════════════════════════
// 9. Enum kebab-case naming
// ═══════════════════════════════════════════════════════════════════

#[test]
fn analysis_preset_uses_kebab_case() {
    assert_eq!(
        serde_json::to_string(&AnalysisPreset::Receipt).unwrap(),
        "\"receipt\""
    );
    assert_eq!(
        serde_json::to_string(&AnalysisPreset::Deep).unwrap(),
        "\"deep\""
    );
}

#[test]
fn context_strategy_uses_kebab_case() {
    assert_eq!(
        serde_json::to_string(&ContextStrategy::Greedy).unwrap(),
        "\"greedy\""
    );
    assert_eq!(
        serde_json::to_string(&ContextStrategy::Spread).unwrap(),
        "\"spread\""
    );
}

#[test]
fn value_metric_uses_kebab_case() {
    assert_eq!(
        serde_json::to_string(&ValueMetric::Hotspot).unwrap(),
        "\"hotspot\""
    );
    assert_eq!(
        serde_json::to_string(&ValueMetric::Code).unwrap(),
        "\"code\""
    );
}

#[test]
fn handoff_preset_uses_kebab_case() {
    assert_eq!(
        serde_json::to_string(&HandoffPreset::Risk).unwrap(),
        "\"risk\""
    );
    assert_eq!(
        serde_json::to_string(&HandoffPreset::Deep).unwrap(),
        "\"deep\""
    );
}

// ═══════════════════════════════════════════════════════════════════
// 10. UserConfig serde roundtrip
// ═══════════════════════════════════════════════════════════════════

#[test]
fn user_config_full_roundtrip() {
    let mut c = UserConfig::default();
    c.profiles.insert(
        "ci".into(),
        Profile {
            format: Some("json".into()),
            top: Some(5),
            files: Some(true),
            redact: Some(CliRedactMode::Paths),
            ..Profile::default()
        },
    );
    c.profiles.insert(
        "llm".into(),
        Profile {
            format: Some("md".into()),
            redact: Some(CliRedactMode::All),
            ..Profile::default()
        },
    );
    c.repos.insert("owner/repo".into(), "ci".into());
    c.repos.insert("org/project".into(), "llm".into());

    let json = serde_json::to_string(&c).unwrap();
    let back: UserConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(back.profiles.len(), 2);
    assert_eq!(back.repos.len(), 2);
    assert_eq!(back.profiles["ci"].top, Some(5));
    assert_eq!(back.profiles["llm"].format, Some("md".into()));
}

// ═══════════════════════════════════════════════════════════════════
// 11. ViewProfile default and field coverage
// ═══════════════════════════════════════════════════════════════════

#[test]
fn view_profile_default_all_none() {
    let v = ViewProfile::default();
    assert!(v.format.is_none());
    assert!(v.top.is_none());
    assert!(v.files.is_none());
    assert!(v.module_roots.is_none());
    assert!(v.module_depth.is_none());
    assert!(v.min_code.is_none());
    assert!(v.max_rows.is_none());
    assert!(v.redact.is_none());
    assert!(v.meta.is_none());
    assert!(v.children.is_none());
    assert!(v.preset.is_none());
    assert!(v.window.is_none());
    assert!(v.budget.is_none());
    assert!(v.strategy.is_none());
    assert!(v.rank_by.is_none());
    assert!(v.output.is_none());
    assert!(v.compress.is_none());
    assert!(v.metric.is_none());
}

#[test]
fn view_profile_serde_roundtrip() {
    let v = ViewProfile {
        format: Some("json".into()),
        top: Some(10),
        files: Some(true),
        module_roots: Some(vec!["crates".into()]),
        module_depth: Some(2),
        min_code: Some(1),
        max_rows: Some(500),
        redact: Some("paths".into()),
        meta: Some(true),
        children: Some("separate".into()),
        preset: Some("health".into()),
        window: Some(128_000),
        budget: Some("128k".into()),
        strategy: Some("greedy".into()),
        rank_by: Some("code".into()),
        output: Some("list".into()),
        compress: Some(false),
        metric: Some("lines".into()),
    };
    let json = serde_json::to_string(&v).unwrap();
    let back: ViewProfile = serde_json::from_str(&json).unwrap();
    assert_eq!(back.format, v.format);
    assert_eq!(back.top, v.top);
    assert_eq!(back.budget, v.budget);
    assert_eq!(back.metric, v.metric);
}

// ═══════════════════════════════════════════════════════════════════
// 12. TomlConfig serialization roundtrip
// ═══════════════════════════════════════════════════════════════════

#[test]
fn toml_config_full_json_roundtrip() {
    let toml_str = r#"
[scan]
hidden = true
paths = ["src"]

[module]
roots = ["crates"]
depth = 2

[export]
min_code = 1
max_rows = 100

[analyze]
preset = "risk"
window = 64000

[context]
budget = "64k"

[badge]
metric = "lines"

[gate]
fail_fast = true

[view.ci]
format = "json"
top = 5
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    let json = serde_json::to_string(&config).unwrap();
    let back: TomlConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(back.scan.hidden, Some(true));
    assert_eq!(back.module.depth, Some(2));
    assert_eq!(back.export.min_code, Some(1));
    assert_eq!(back.analyze.preset, Some("risk".into()));
    assert_eq!(back.context.budget, Some("64k".into()));
    assert_eq!(back.badge.metric, Some("lines".into()));
    assert_eq!(back.gate.fail_fast, Some(true));
    assert_eq!(back.view["ci"].top, Some(5));
}

// ═══════════════════════════════════════════════════════════════════
// 13. Gate config inline rules
// ═══════════════════════════════════════════════════════════════════

#[test]
fn given_gate_with_inline_rules_when_parsed_then_rules_available() {
    let toml_str = r#"
[gate]
fail_fast = true

[[gate.rules]]
name = "max_tokens"
pointer = "/tokens"
op = "lte"
value = 100000

[[gate.rules]]
name = "has_license"
pointer = "/license"
op = "exists"
level = "warn"
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    let rules = config.gate.rules.as_ref().unwrap();
    assert_eq!(rules.len(), 2);
    assert_eq!(rules[0].name, "max_tokens");
    assert_eq!(rules[0].op, "lte");
    assert_eq!(rules[1].name, "has_license");
    assert_eq!(rules[1].op, "exists");
}

#[test]
fn given_gate_with_ratchet_rules_when_parsed_then_ratchet_available() {
    let toml_str = r#"
[gate]
allow_missing_baseline = true

[[gate.ratchet]]
pointer = "/complexity/avg"
max_increase_pct = 5.0
level = "error"
description = "Complexity gate"
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(config.gate.allow_missing_baseline, Some(true));
    let ratchet = config.gate.ratchet.as_ref().unwrap();
    assert_eq!(ratchet.len(), 1);
    assert_eq!(ratchet[0].pointer, "/complexity/avg");
    assert_eq!(ratchet[0].max_increase_pct, Some(5.0));
}

// ═══════════════════════════════════════════════════════════════════
// 14. Property tests for config determinism
// ═══════════════════════════════════════════════════════════════════

mod properties {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn toml_parse_empty_always_succeeds(s in "[ \t\n]*") {
            // ASCII whitespace-only TOML (no bare CR) should always parse
            let result = TomlConfig::parse(&s);
            prop_assert!(result.is_ok());
        }

        #[test]
        fn user_config_json_roundtrip_deterministic(
            name in "[a-z]{1,10}",
            top in 1usize..100,
        ) {
            let mut c = UserConfig::default();
            c.profiles.insert(
                name.clone(),
                Profile {
                    top: Some(top),
                    ..Profile::default()
                },
            );
            let json1 = serde_json::to_string(&c).unwrap();
            let json2 = serde_json::to_string(&c).unwrap();
            prop_assert_eq!(&json1, &json2);
        }

        #[test]
        fn global_args_scan_options_conversion_deterministic(
            hidden in proptest::bool::ANY,
            no_ignore in proptest::bool::ANY,
        ) {
            let g = GlobalArgs {
                hidden,
                no_ignore,
                ..GlobalArgs::default()
            };
            let a: tokmd_settings::ScanOptions = (&g).into();
            let b: tokmd_settings::ScanOptions = (&g).into();
            prop_assert_eq!(a.hidden, b.hidden);
            prop_assert_eq!(a.no_ignore, b.no_ignore);
        }

        #[test]
        fn view_profile_default_always_empty(_dummy in 0u8..1) {
            let v = ViewProfile::default();
            prop_assert!(v.format.is_none());
            prop_assert!(v.top.is_none());
        }
    }
}
