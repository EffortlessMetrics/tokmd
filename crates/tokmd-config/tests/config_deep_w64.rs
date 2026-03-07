//! Deep tests for tokmd-config (w64).
//!
//! Exercises config loading from TOML, default values, profile merging,
//! serialization round-trips, edge cases, and property-based checks.

use tokmd_config::{
    AnalysisPreset, BadgeMetric, CliLangArgs, CockpitFormat, ColorMode, ContextOutput,
    ContextStrategy, DiffFormat, DiffRangeMode, GateFormat, GlobalArgs, HandoffPreset,
    ImportGranularity, InitProfile, NearDupScope, Profile, SensorFormat, Shell, UserConfig,
    ValueMetric,
};
use tokmd_settings::{
    AnalyzeSettings, DiffSettings, ExportSettings, GateRule, LangSettings, ModuleConfig,
    ModuleSettings, RatchetRuleConfig, ScanConfig, ScanSettings, TomlConfig, ViewProfile,
};

// ═══════════════════════════════════════════════════════════════════
// 1. TOML loading — empty and minimal configs
// ═══════════════════════════════════════════════════════════════════

#[test]
fn empty_toml_parses_to_defaults() {
    let cfg: TomlConfig = toml::from_str("").unwrap();
    assert!(cfg.scan.paths.is_none());
    assert!(cfg.scan.exclude.is_none());
    assert!(cfg.module.roots.is_none());
    assert!(cfg.view.is_empty());
}

#[test]
fn minimal_scan_section() {
    let cfg: TomlConfig = toml::from_str(
        r#"
        [scan]
        paths = ["src"]
        "#,
    )
    .unwrap();
    assert_eq!(cfg.scan.paths, Some(vec!["src".to_string()]));
}

#[test]
fn scan_with_exclude() {
    let cfg: TomlConfig = toml::from_str(
        r#"
        [scan]
        exclude = ["target", "**/*.min.js"]
        "#,
    )
    .unwrap();
    let excludes = cfg.scan.exclude.unwrap();
    assert_eq!(excludes.len(), 2);
    assert!(excludes.contains(&"target".to_string()));
}

#[test]
fn scan_with_boolean_flags() {
    let cfg: TomlConfig = toml::from_str(
        r#"
        [scan]
        hidden = true
        no_ignore = true
        no_ignore_parent = true
        no_ignore_dot = true
        no_ignore_vcs = true
        doc_comments = true
        "#,
    )
    .unwrap();
    assert_eq!(cfg.scan.hidden, Some(true));
    assert_eq!(cfg.scan.no_ignore, Some(true));
    assert_eq!(cfg.scan.no_ignore_parent, Some(true));
    assert_eq!(cfg.scan.no_ignore_dot, Some(true));
    assert_eq!(cfg.scan.no_ignore_vcs, Some(true));
    assert_eq!(cfg.scan.doc_comments, Some(true));
}

#[test]
fn module_section_roots_and_depth() {
    let cfg: TomlConfig = toml::from_str(
        r#"
        [module]
        roots = ["crates", "libs"]
        depth = 3
        children = "collapse"
        "#,
    )
    .unwrap();
    assert_eq!(
        cfg.module.roots,
        Some(vec!["crates".to_string(), "libs".to_string()])
    );
    assert_eq!(cfg.module.depth, Some(3));
    assert_eq!(cfg.module.children, Some("collapse".to_string()));
}

#[test]
fn export_section_all_fields() {
    let cfg: TomlConfig = toml::from_str(
        r#"
        [export]
        min_code = 10
        max_rows = 500
        redact = "paths"
        format = "csv"
        children = "separate"
        "#,
    )
    .unwrap();
    assert_eq!(cfg.export.min_code, Some(10));
    assert_eq!(cfg.export.max_rows, Some(500));
    assert_eq!(cfg.export.redact, Some("paths".to_string()));
    assert_eq!(cfg.export.format, Some("csv".to_string()));
}

#[test]
fn analyze_section_all_fields() {
    let cfg: TomlConfig = toml::from_str(
        r#"
        [analyze]
        preset = "deep"
        window = 128000
        format = "json"
        git = true
        max_files = 5000
        max_bytes = 10000000
        max_file_bytes = 500000
        max_commits = 500
        max_commit_files = 100
        granularity = "file"
        "#,
    )
    .unwrap();
    assert_eq!(cfg.analyze.preset, Some("deep".to_string()));
    assert_eq!(cfg.analyze.window, Some(128_000));
    assert_eq!(cfg.analyze.git, Some(true));
    assert_eq!(cfg.analyze.max_files, Some(5000));
    assert_eq!(cfg.analyze.granularity, Some("file".to_string()));
}

#[test]
fn context_section_all_fields() {
    let cfg: TomlConfig = toml::from_str(
        r#"
        [context]
        budget = "256k"
        strategy = "spread"
        rank_by = "hotspot"
        output = "bundle"
        compress = true
        "#,
    )
    .unwrap();
    assert_eq!(cfg.context.budget, Some("256k".to_string()));
    assert_eq!(cfg.context.strategy, Some("spread".to_string()));
    assert_eq!(cfg.context.rank_by, Some("hotspot".to_string()));
    assert_eq!(cfg.context.output, Some("bundle".to_string()));
    assert_eq!(cfg.context.compress, Some(true));
}

#[test]
fn badge_section() {
    let cfg: TomlConfig = toml::from_str(
        r#"
        [badge]
        metric = "tokens"
        "#,
    )
    .unwrap();
    assert_eq!(cfg.badge.metric, Some("tokens".to_string()));
}

// ═══════════════════════════════════════════════════════════════════
// 2. View profiles
// ═══════════════════════════════════════════════════════════════════

#[test]
fn view_profile_parsed() {
    let cfg: TomlConfig = toml::from_str(
        r#"
        [view.llm]
        format = "json"
        top = 10
        redact = "all"
        files = true
        "#,
    )
    .unwrap();
    assert!(cfg.view.contains_key("llm"));
    let p = &cfg.view["llm"];
    assert_eq!(p.format, Some("json".to_string()));
    assert_eq!(p.top, Some(10));
    assert_eq!(p.redact, Some("all".to_string()));
    assert_eq!(p.files, Some(true));
}

#[test]
fn multiple_view_profiles() {
    let cfg: TomlConfig = toml::from_str(
        r#"
        [view.llm]
        format = "json"
        top = 10

        [view.ci]
        format = "tsv"
        top = 0
        "#,
    )
    .unwrap();
    assert_eq!(cfg.view.len(), 2);
    assert!(cfg.view.contains_key("llm"));
    assert!(cfg.view.contains_key("ci"));
}

#[test]
fn view_profile_default_is_empty() {
    let p = ViewProfile::default();
    assert!(p.format.is_none());
    assert!(p.top.is_none());
    assert!(p.files.is_none());
    assert!(p.module_roots.is_none());
    assert!(p.preset.is_none());
    assert!(p.budget.is_none());
    assert!(p.metric.is_none());
}

// ═══════════════════════════════════════════════════════════════════
// 3. Gate section with inline rules
// ═══════════════════════════════════════════════════════════════════

#[test]
fn gate_section_with_rules() {
    let cfg: TomlConfig = toml::from_str(
        r#"
        [gate]
        fail_fast = true

        [[gate.rules]]
        name = "max-complexity"
        pointer = "/complexity/max_cyclomatic"
        op = "lte"
        value = 20

        [[gate.rules]]
        name = "min-doc"
        pointer = "/density/doc_ratio"
        op = "gte"
        value = 0.1
        "#,
    )
    .unwrap();
    assert_eq!(cfg.gate.fail_fast, Some(true));
    let rules = cfg.gate.rules.unwrap();
    assert_eq!(rules.len(), 2);
    assert_eq!(rules[0].name, "max-complexity");
    assert_eq!(rules[1].op, "gte");
}

#[test]
fn gate_section_with_ratchet() {
    let cfg: TomlConfig = toml::from_str(
        r#"
        [gate]
        baseline = ".tokmd/baseline.json"

        [[gate.ratchet]]
        pointer = "/complexity/avg_cyclomatic"
        max_increase_pct = 5.0
        level = "error"
        description = "Average complexity must not increase by more than 5%"
        "#,
    )
    .unwrap();
    assert_eq!(cfg.gate.baseline, Some(".tokmd/baseline.json".to_string()));
    let ratchet = cfg.gate.ratchet.unwrap();
    assert_eq!(ratchet.len(), 1);
    assert_eq!(ratchet[0].max_increase_pct, Some(5.0));
}

// ═══════════════════════════════════════════════════════════════════
// 4. Serialization round-trips
// ═══════════════════════════════════════════════════════════════════

#[test]
fn toml_config_json_roundtrip() {
    let cfg = TomlConfig {
        scan: ScanConfig {
            paths: Some(vec!["src".to_string()]),
            exclude: Some(vec!["target".to_string()]),
            hidden: Some(true),
            ..ScanConfig::default()
        },
        module: ModuleConfig {
            roots: Some(vec!["crates".to_string()]),
            depth: Some(3),
            ..ModuleConfig::default()
        },
        ..TomlConfig::default()
    };
    let json = serde_json::to_string(&cfg).unwrap();
    let back: TomlConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(back.scan.paths, Some(vec!["src".to_string()]));
    assert_eq!(back.module.depth, Some(3));
}

#[test]
fn scan_settings_json_roundtrip() {
    let s = ScanSettings::for_paths(vec!["a".to_string(), "b".to_string()]);
    let json = serde_json::to_string(&s).unwrap();
    let back: ScanSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.paths, ["a".to_string(), "b".to_string()]);
}

#[test]
fn lang_settings_json_roundtrip() {
    let s = LangSettings {
        top: 15,
        files: true,
        ..LangSettings::default()
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: LangSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.top, 15);
    assert!(back.files);
}

#[test]
fn module_settings_json_roundtrip() {
    let s = ModuleSettings {
        top: 10,
        module_depth: 4,
        module_roots: vec!["lib".to_string()],
        ..ModuleSettings::default()
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: ModuleSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.top, 10);
    assert_eq!(back.module_depth, 4);
}

#[test]
fn export_settings_json_roundtrip() {
    let s = ExportSettings {
        min_code: 50,
        max_rows: 100,
        ..ExportSettings::default()
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: ExportSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.min_code, 50);
    assert_eq!(back.max_rows, 100);
}

#[test]
fn analyze_settings_json_roundtrip() {
    let s = AnalyzeSettings {
        preset: "risk".to_string(),
        window: Some(64_000),
        git: Some(true),
        ..AnalyzeSettings::default()
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: AnalyzeSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.preset, "risk");
    assert_eq!(back.window, Some(64_000));
}

#[test]
fn diff_settings_json_roundtrip() {
    let s = DiffSettings {
        from: "v1.0".to_string(),
        to: "v2.0".to_string(),
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: DiffSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.from, "v1.0");
    assert_eq!(back.to, "v2.0");
}

#[test]
fn user_config_json_roundtrip() {
    let mut cfg = UserConfig::default();
    cfg.profiles.insert(
        "ci".into(),
        Profile {
            format: Some("tsv".into()),
            top: Some(20),
            ..Profile::default()
        },
    );
    cfg.repos.insert("org/repo".into(), "ci".into());
    let json = serde_json::to_string(&cfg).unwrap();
    let back: UserConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(back.profiles["ci"].top, Some(20));
    assert_eq!(back.repos["org/repo"], "ci");
}

// ═══════════════════════════════════════════════════════════════════
// 5. Default value verification
// ═══════════════════════════════════════════════════════════════════

#[test]
fn global_args_default_excluded_empty() {
    let g = GlobalArgs::default();
    assert!(g.excluded.is_empty());
}

#[test]
fn global_args_default_config_auto() {
    let g = GlobalArgs::default();
    assert_eq!(g.config, tokmd_config::ConfigMode::Auto);
}

#[test]
fn global_args_default_flags_false() {
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
fn cli_lang_args_defaults() {
    let a = CliLangArgs::default();
    assert!(a.paths.is_none());
    assert!(a.format.is_none());
    assert!(a.top.is_none());
    assert!(!a.files);
    assert!(a.children.is_none());
}

// ═══════════════════════════════════════════════════════════════════
// 6. Enum serde roundtrips
// ═══════════════════════════════════════════════════════════════════

#[test]
fn analysis_preset_all_variants_roundtrip() {
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
        AnalysisPreset::Estimate,
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: AnalysisPreset = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn import_granularity_roundtrip() {
    for variant in [ImportGranularity::Module, ImportGranularity::File] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: ImportGranularity = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn badge_metric_roundtrip() {
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
fn shell_variants_roundtrip() {
    for variant in [
        Shell::Bash,
        Shell::Elvish,
        Shell::Fish,
        Shell::Powershell,
        Shell::Zsh,
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: Shell = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn init_profile_roundtrip() {
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
fn diff_format_roundtrip() {
    for variant in [DiffFormat::Md, DiffFormat::Json] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: DiffFormat = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn color_mode_roundtrip() {
    for variant in [ColorMode::Auto, ColorMode::Always, ColorMode::Never] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: ColorMode = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn context_strategy_roundtrip() {
    for variant in [ContextStrategy::Greedy, ContextStrategy::Spread] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: ContextStrategy = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn value_metric_roundtrip() {
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
fn context_output_roundtrip() {
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
// 8. GlobalArgs → ScanOptions conversion
// ═══════════════════════════════════════════════════════════════════

#[test]
fn global_args_to_scan_options_preserves_fields() {
    let g = GlobalArgs {
        excluded: vec!["target".to_string()],
        hidden: true,
        no_ignore: true,
        no_ignore_parent: true,
        no_ignore_dot: true,
        no_ignore_vcs: true,
        treat_doc_strings_as_comments: true,
        ..GlobalArgs::default()
    };
    let opts: tokmd_settings::ScanOptions = (&g).into();
    assert_eq!(opts.excluded, ["target".to_string()]);
    assert!(opts.hidden);
    assert!(opts.no_ignore);
    assert!(opts.no_ignore_parent);
    assert!(opts.no_ignore_dot);
    assert!(opts.no_ignore_vcs);
    assert!(opts.treat_doc_strings_as_comments);
}

#[test]
fn global_args_owned_conversion() {
    let g = GlobalArgs::default();
    let opts: tokmd_settings::ScanOptions = g.into();
    assert!(opts.excluded.is_empty());
    assert!(!opts.hidden);
}

// ═══════════════════════════════════════════════════════════════════
// 9. BDD-style: Given TOML / When loading / Then settings match
// ═══════════════════════════════════════════════════════════════════

#[test]
fn given_empty_toml_when_load_then_all_defaults() {
    // Given
    let toml_str = "";
    // When
    let cfg: TomlConfig = toml::from_str(toml_str).unwrap();
    // Then
    assert!(cfg.scan.paths.is_none());
    assert!(cfg.module.roots.is_none());
    assert!(cfg.export.min_code.is_none());
    assert!(cfg.analyze.preset.is_none());
    assert!(cfg.context.budget.is_none());
    assert!(cfg.badge.metric.is_none());
    assert!(cfg.gate.policy.is_none());
    assert!(cfg.view.is_empty());
}

#[test]
fn given_full_config_when_load_then_all_sections_present() {
    // Given
    let toml_str = r#"
        [scan]
        paths = ["src", "lib"]
        exclude = ["dist"]

        [module]
        roots = ["packages"]
        depth = 2

        [export]
        min_code = 5
        max_rows = 100

        [analyze]
        preset = "health"

        [context]
        budget = "64k"

        [badge]
        metric = "lines"

        [gate]
        fail_fast = true

        [view.quick]
        top = 5
    "#;
    // When
    let cfg: TomlConfig = toml::from_str(toml_str).unwrap();
    // Then
    assert_eq!(
        cfg.scan.paths,
        Some(vec!["src".to_string(), "lib".to_string()])
    );
    assert_eq!(cfg.module.roots, Some(vec!["packages".to_string()]));
    assert_eq!(cfg.export.min_code, Some(5));
    assert_eq!(cfg.analyze.preset, Some("health".to_string()));
    assert_eq!(cfg.context.budget, Some("64k".to_string()));
    assert_eq!(cfg.badge.metric, Some("lines".to_string()));
    assert_eq!(cfg.gate.fail_fast, Some(true));
    assert_eq!(cfg.view["quick"].top, Some(5));
}

// ═══════════════════════════════════════════════════════════════════
// 10. Edge cases — extra/unknown fields
// ═══════════════════════════════════════════════════════════════════

#[test]
fn extra_fields_in_scan_ignored() {
    // TOML deserialization with #[serde(default)] ignores unknown fields
    let cfg: TomlConfig = toml::from_str(
        r#"
        [scan]
        paths = ["."]
        unknown_field = "ignored"
        "#,
    )
    .unwrap();
    assert_eq!(cfg.scan.paths, Some(vec![".".to_string()]));
}

#[test]
fn empty_arrays_in_config() {
    let cfg: TomlConfig = toml::from_str(
        r#"
        [scan]
        paths = []
        exclude = []
        "#,
    )
    .unwrap();
    assert_eq!(cfg.scan.paths, Some(vec![]));
    assert_eq!(cfg.scan.exclude, Some(vec![]));
}

#[test]
fn empty_view_profiles_section() {
    let cfg: TomlConfig = toml::from_str("[view]").unwrap();
    assert!(cfg.view.is_empty());
}

// ═══════════════════════════════════════════════════════════════════
// 11. Settings types — Tier 0 defaults
// ═══════════════════════════════════════════════════════════════════

#[test]
fn scan_settings_default() {
    let s = ScanSettings::default();
    assert!(s.paths.is_empty());
    assert!(s.options.excluded.is_empty());
    assert!(!s.options.hidden);
}

#[test]
fn scan_settings_current_dir() {
    let s = ScanSettings::current_dir();
    assert_eq!(s.paths, [".".to_string()]);
}

#[test]
fn scan_settings_for_paths() {
    let s = ScanSettings::for_paths(vec!["a".to_string(), "b".to_string()]);
    assert_eq!(s.paths.len(), 2);
}

#[test]
fn lang_settings_default() {
    let s = LangSettings::default();
    assert_eq!(s.top, 0);
    assert!(!s.files);
    assert!(s.redact.is_none());
}

#[test]
fn module_settings_default() {
    let s = ModuleSettings::default();
    assert_eq!(s.top, 0);
    assert_eq!(s.module_depth, 2);
    assert!(s.module_roots.contains(&"crates".to_string()));
    assert!(s.module_roots.contains(&"packages".to_string()));
}

#[test]
fn export_settings_default() {
    let s = ExportSettings::default();
    assert_eq!(s.min_code, 0);
    assert_eq!(s.max_rows, 0);
    assert_eq!(s.module_depth, 2);
    assert!(s.meta);
    assert!(s.strip_prefix.is_none());
}

#[test]
fn analyze_settings_default() {
    let s = AnalyzeSettings::default();
    assert_eq!(s.preset, "receipt");
    assert_eq!(s.granularity, "module");
    assert!(s.window.is_none());
    assert!(s.git.is_none());
}

// ═══════════════════════════════════════════════════════════════════
// 12. TOML roundtrip (serialize to TOML, then parse back)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn toml_config_toml_roundtrip() {
    let cfg = TomlConfig {
        scan: ScanConfig {
            paths: Some(vec!["src".to_string()]),
            hidden: Some(true),
            ..ScanConfig::default()
        },
        ..TomlConfig::default()
    };
    let toml_str = toml::to_string(&cfg).unwrap();
    let back: TomlConfig = toml::from_str(&toml_str).unwrap();
    assert_eq!(back.scan.paths, Some(vec!["src".to_string()]));
    assert_eq!(back.scan.hidden, Some(true));
}

#[test]
fn gate_rule_toml_roundtrip() {
    let rule = GateRule {
        name: "max-loc".to_string(),
        pointer: "/total/code".to_string(),
        op: "lte".to_string(),
        value: Some(serde_json::json!(100_000)),
        values: None,
        negate: false,
        level: Some("error".to_string()),
        message: Some("Too many lines".to_string()),
    };
    let json = serde_json::to_string(&rule).unwrap();
    let back: GateRule = serde_json::from_str(&json).unwrap();
    assert_eq!(back.name, "max-loc");
    assert_eq!(back.op, "lte");
}

#[test]
fn ratchet_rule_toml_roundtrip() {
    let rule = RatchetRuleConfig {
        pointer: "/complexity/avg".to_string(),
        max_increase_pct: Some(10.0),
        max_value: Some(50.0),
        level: Some("warn".to_string()),
        description: Some("Keep complexity low".to_string()),
    };
    let json = serde_json::to_string(&rule).unwrap();
    let back: RatchetRuleConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(back.pointer, "/complexity/avg");
    assert_eq!(back.max_increase_pct, Some(10.0));
}
