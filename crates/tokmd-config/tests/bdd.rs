//! BDD-style scenario tests for tokmd-config.
//!
//! Each test follows Given/When/Then structure covering:
//! - TomlConfig parsing (valid, invalid, defaults, sections)
//! - UserConfig profile lookup
//! - GlobalArgs → ScanOptions conversion
//! - ViewProfile defaults and field handling
//! - Enum variant coverage

use std::collections::BTreeMap;

use tokmd_config::{
    AnalysisPreset, BadgeMetric, ChildrenMode, ColorMode, ConfigMode, DiffFormat, ExportFormat,
    GlobalArgs, InitProfile, Profile, RedactMode, TableFormat, TomlConfig, UserConfig, ViewProfile,
};

// ============================================================================
// Scenario: TomlConfig parsing — valid configurations
// ============================================================================

mod given_valid_toml {
    use super::*;

    #[test]
    fn when_empty_string_parsed_then_all_defaults_applied() {
        let config = TomlConfig::parse("").expect("empty TOML should parse");

        assert_eq!(config.scan.hidden, None);
        assert_eq!(config.scan.no_ignore, None);
        assert_eq!(config.module.depth, None);
        assert_eq!(config.module.roots, None);
        assert_eq!(config.export.min_code, None);
        assert_eq!(config.export.max_rows, None);
    }

    #[test]
    fn when_scan_section_present_then_scan_fields_populated() {
        let toml_str = r#"
[scan]
hidden = true
no_ignore = true
no_ignore_parent = false
no_ignore_vcs = true
doc_comments = true
"#;
        let config = TomlConfig::parse(toml_str).unwrap();

        assert_eq!(config.scan.hidden, Some(true));
        assert_eq!(config.scan.no_ignore, Some(true));
        assert_eq!(config.scan.no_ignore_parent, Some(false));
        assert_eq!(config.scan.no_ignore_vcs, Some(true));
        assert_eq!(config.scan.doc_comments, Some(true));
    }

    #[test]
    fn when_module_section_present_then_roots_and_depth_loaded() {
        let toml_str = r#"
[module]
roots = ["crates", "packages"]
depth = 3
"#;
        let config = TomlConfig::parse(toml_str).unwrap();

        assert_eq!(
            config.module.roots,
            Some(vec!["crates".to_string(), "packages".to_string()])
        );
        assert_eq!(config.module.depth, Some(3));
    }

    #[test]
    fn when_export_section_present_then_all_fields_loaded() {
        let toml_str = r#"
[export]
min_code = 5
max_rows = 200
redact = "paths"
"#;
        let config = TomlConfig::parse(toml_str).unwrap();

        assert_eq!(config.export.min_code, Some(5));
        assert_eq!(config.export.max_rows, Some(200));
        assert_eq!(config.export.redact, Some("paths".to_string()));
    }

    #[test]
    fn when_analyze_section_present_then_preset_and_window_loaded() {
        let toml_str = r#"
[analyze]
preset = "deep"
window = 128000
max_files = 500
max_bytes = 1000000
max_commits = 200
"#;
        let config = TomlConfig::parse(toml_str).unwrap();

        assert_eq!(config.analyze.preset, Some("deep".to_string()));
        assert_eq!(config.analyze.window, Some(128000));
        assert_eq!(config.analyze.max_files, Some(500));
        assert_eq!(config.analyze.max_bytes, Some(1000000));
        assert_eq!(config.analyze.max_commits, Some(200));
    }

    #[test]
    fn when_view_profiles_present_then_multiple_profiles_loaded() {
        let toml_str = r#"
[view.llm]
format = "json"
redact = "all"
top = 10

[view.ci]
format = "tsv"
compress = true
"#;
        let config = TomlConfig::parse(toml_str).unwrap();

        assert!(config.view.contains_key("llm"));
        assert!(config.view.contains_key("ci"));
        let llm = &config.view["llm"];
        assert_eq!(llm.format, Some("json".to_string()));
        assert_eq!(llm.redact, Some("all".to_string()));
        assert_eq!(llm.top, Some(10));
        let ci = &config.view["ci"];
        assert_eq!(ci.format, Some("tsv".to_string()));
        assert_eq!(ci.compress, Some(true));
    }

    #[test]
    fn when_gate_section_present_then_fail_fast_loaded() {
        let toml_str = r#"
[gate]
fail_fast = true
"#;
        let config = TomlConfig::parse(toml_str).unwrap();

        assert_eq!(config.gate.fail_fast, Some(true));
    }

    #[test]
    fn when_context_section_present_then_budget_and_strategy_loaded() {
        let toml_str = r#"
[context]
budget = "64k"
strategy = "spread"
compress = true
"#;
        let config = TomlConfig::parse(toml_str).unwrap();

        assert_eq!(config.context.budget, Some("64k".to_string()));
        assert_eq!(config.context.strategy, Some("spread".to_string()));
        assert_eq!(config.context.compress, Some(true));
    }
}

// ============================================================================
// Scenario: TomlConfig parsing — invalid inputs
// ============================================================================

mod given_invalid_toml {
    use super::*;

    #[test]
    fn when_malformed_syntax_then_parse_returns_error() {
        let result = TomlConfig::parse("[scan\nhidden = true");
        assert!(result.is_err(), "malformed TOML should fail to parse");
    }

    #[test]
    fn when_wrong_type_for_boolean_then_parse_returns_error() {
        let result = TomlConfig::parse("[scan]\nhidden = \"not_a_bool\"");
        assert!(
            result.is_err(),
            "string where bool expected should fail to parse"
        );
    }

    #[test]
    fn when_wrong_type_for_number_then_parse_returns_error() {
        let result = TomlConfig::parse("[module]\ndepth = \"three\"");
        assert!(
            result.is_err(),
            "string where number expected should fail to parse"
        );
    }
}

// ============================================================================
// Scenario: UserConfig profile lookup
// ============================================================================

mod given_user_config {
    use super::*;

    #[test]
    fn when_profile_exists_then_lookup_returns_it() {
        let mut profiles = BTreeMap::new();
        profiles.insert(
            "llm_safe".to_string(),
            Profile {
                format: Some("json".to_string()),
                redact: Some(RedactMode::All),
                top: Some(10),
                ..Default::default()
            },
        );
        let config = UserConfig {
            profiles,
            repos: BTreeMap::new(),
        };

        let p = config.profiles.get("llm_safe").unwrap();
        assert_eq!(p.format, Some("json".to_string()));
        assert_eq!(p.top, Some(10));
    }

    #[test]
    fn when_profile_missing_then_lookup_returns_none() {
        let config = UserConfig::default();
        assert!(config.profiles.get("nonexistent").is_none());
    }

    #[test]
    fn when_repo_mapped_to_profile_then_mapping_is_retrievable() {
        let mut repos = BTreeMap::new();
        repos.insert("org/repo".to_string(), "ci".to_string());
        let config = UserConfig {
            profiles: BTreeMap::new(),
            repos,
        };

        assert_eq!(config.repos.get("org/repo"), Some(&"ci".to_string()));
        assert_eq!(config.repos.get("other/repo"), None);
    }
}

// ============================================================================
// Scenario: GlobalArgs → ScanOptions conversion
// ============================================================================

mod given_global_args {
    use super::*;
    use tokmd_settings::ScanOptions;

    #[test]
    fn when_defaults_then_scan_options_are_all_false() {
        let args = GlobalArgs::default();
        let opts: ScanOptions = (&args).into();

        assert!(opts.excluded.is_empty());
        assert!(!opts.hidden);
        assert!(!opts.no_ignore);
        assert!(!opts.treat_doc_strings_as_comments);
    }

    #[test]
    fn when_all_flags_set_then_all_options_propagated() {
        let args = GlobalArgs {
            excluded: vec!["target".into(), "node_modules".into()],
            hidden: true,
            no_ignore: true,
            no_ignore_parent: true,
            no_ignore_dot: true,
            no_ignore_vcs: true,
            treat_doc_strings_as_comments: true,
            config: ConfigMode::None,
            ..Default::default()
        };
        let opts: ScanOptions = (&args).into();

        assert_eq!(opts.excluded.len(), 2);
        assert!(opts.hidden);
        assert!(opts.no_ignore);
        assert!(opts.no_ignore_parent);
        assert!(opts.no_ignore_dot);
        assert!(opts.no_ignore_vcs);
        assert!(opts.treat_doc_strings_as_comments);
        assert_eq!(opts.config, ConfigMode::None);
    }
}

// ============================================================================
// Scenario: ViewProfile defaults
// ============================================================================

mod given_view_profile {
    use super::*;

    #[test]
    fn when_default_then_all_fields_are_none() {
        let p = ViewProfile::default();
        assert_eq!(p.format, None);
        assert_eq!(p.top, None);
        assert_eq!(p.files, None);
        assert_eq!(p.module_roots, None);
        assert_eq!(p.module_depth, None);
        assert_eq!(p.min_code, None);
        assert_eq!(p.max_rows, None);
        assert_eq!(p.redact, None);
        assert_eq!(p.meta, None);
        assert_eq!(p.children, None);
        assert_eq!(p.preset, None);
        assert_eq!(p.budget, None);
        assert_eq!(p.strategy, None);
        assert_eq!(p.compress, None);
        assert_eq!(p.metric, None);
    }
}

// ============================================================================
// Scenario: Enum variant coverage
// ============================================================================

mod given_config_enums {
    use super::*;

    #[test]
    fn when_table_format_variants_checked_then_all_exist() {
        let _md = TableFormat::Md;
        let _tsv = TableFormat::Tsv;
        let _json = TableFormat::Json;
    }

    #[test]
    fn when_export_format_variants_checked_then_all_exist() {
        let _csv = ExportFormat::Csv;
        let _jsonl = ExportFormat::Jsonl;
        let _json = ExportFormat::Json;
        let _cdx = ExportFormat::Cyclonedx;
    }

    #[test]
    fn when_redact_mode_variants_checked_then_all_exist() {
        let _none = RedactMode::None;
        let _paths = RedactMode::Paths;
        let _all = RedactMode::All;
    }

    #[test]
    fn when_children_mode_variants_checked_then_all_exist() {
        let _collapse = ChildrenMode::Collapse;
        let _separate = ChildrenMode::Separate;
    }

    #[test]
    fn when_analysis_preset_variants_checked_then_all_exist() {
        let _receipt = AnalysisPreset::Receipt;
        let _health = AnalysisPreset::Health;
        let _risk = AnalysisPreset::Risk;
        let _deep = AnalysisPreset::Deep;
    }

    #[test]
    fn when_badge_metric_variants_checked_then_all_exist() {
        let _lines = BadgeMetric::Lines;
        let _tokens = BadgeMetric::Tokens;
        let _bytes = BadgeMetric::Bytes;
        let _doc = BadgeMetric::Doc;
        let _blank = BadgeMetric::Blank;
        let _hotspot = BadgeMetric::Hotspot;
    }

    #[test]
    fn when_diff_format_variants_checked_then_all_exist() {
        let _md = DiffFormat::Md;
        let _json = DiffFormat::Json;
    }

    #[test]
    fn when_color_mode_variants_checked_then_all_exist() {
        let _auto = ColorMode::Auto;
        let _always = ColorMode::Always;
        let _never = ColorMode::Never;
    }

    #[test]
    fn when_config_mode_variants_checked_then_all_exist() {
        let _auto = ConfigMode::Auto;
        let _none = ConfigMode::None;
    }

    #[test]
    fn when_init_profile_variants_checked_then_all_exist() {
        let _default = InitProfile::Default;
        let _rust = InitProfile::Rust;
        let _node = InitProfile::Node;
        let _mono = InitProfile::Mono;
        let _python = InitProfile::Python;
    }
}
