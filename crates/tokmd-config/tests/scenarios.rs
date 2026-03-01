//! BDD-style scenario tests for tokmd-config.
//!
//! Each test follows Given/When/Then structure covering:
//! - Profile merging semantics
//! - CLI override application (GlobalArgs → ScanOptions)
//! - UserConfig profile lookup and edge cases
//! - Clap parsing edge cases

use std::collections::BTreeMap;
use tokmd_config::{CliLangArgs, GlobalArgs, Profile, TomlConfig, UserConfig, ViewProfile};

// =========================================================================
// Scenario: GlobalArgs → ScanOptions conversion
// =========================================================================

mod scan_options_conversion {
    use super::*;
    use tokmd_settings::ScanOptions;

    #[test]
    fn default_global_args_produce_default_scan_options() {
        // Given default GlobalArgs
        let args = GlobalArgs::default();

        // When converted to ScanOptions
        let opts: ScanOptions = (&args).into();

        // Then all fields are at their defaults
        assert!(opts.excluded.is_empty());
        assert!(!opts.hidden);
        assert!(!opts.no_ignore);
        assert!(!opts.no_ignore_parent);
        assert!(!opts.no_ignore_dot);
        assert!(!opts.no_ignore_vcs);
        assert!(!opts.treat_doc_strings_as_comments);
    }

    #[test]
    fn excluded_patterns_are_forwarded() {
        // Given GlobalArgs with exclude patterns
        let args = GlobalArgs {
            excluded: vec!["target".into(), "**/*.min.js".into()],
            ..Default::default()
        };

        // When converted
        let opts: ScanOptions = (&args).into();

        // Then patterns are preserved
        assert_eq!(opts.excluded, vec!["target", "**/*.min.js"]);
    }

    #[test]
    fn all_boolean_flags_are_forwarded() {
        // Given GlobalArgs with all booleans enabled
        let args = GlobalArgs {
            hidden: true,
            no_ignore: true,
            no_ignore_parent: true,
            no_ignore_dot: true,
            no_ignore_vcs: true,
            treat_doc_strings_as_comments: true,
            ..Default::default()
        };

        // When converted
        let opts: ScanOptions = (&args).into();

        // Then all flags are set
        assert!(opts.hidden);
        assert!(opts.no_ignore);
        assert!(opts.no_ignore_parent);
        assert!(opts.no_ignore_dot);
        assert!(opts.no_ignore_vcs);
        assert!(opts.treat_doc_strings_as_comments);
    }

    #[test]
    fn owned_conversion_matches_borrowed() {
        // Given the same GlobalArgs
        let args = GlobalArgs {
            excluded: vec!["node_modules".into()],
            hidden: true,
            no_ignore_vcs: true,
            ..Default::default()
        };

        // When converted via reference and owned
        let opts_ref: ScanOptions = (&args).into();
        let opts_owned: ScanOptions = args.into();

        // Then both produce the same result
        assert_eq!(opts_ref.excluded, opts_owned.excluded);
        assert_eq!(opts_ref.hidden, opts_owned.hidden);
        assert_eq!(opts_ref.no_ignore_vcs, opts_owned.no_ignore_vcs);
    }

    #[test]
    fn config_mode_is_forwarded() {
        use tokmd_config::ConfigMode;

        let args = GlobalArgs {
            config: ConfigMode::None,
            ..Default::default()
        };
        let opts: ScanOptions = (&args).into();
        assert_eq!(opts.config, ConfigMode::None);

        let args2 = GlobalArgs {
            config: ConfigMode::Auto,
            ..Default::default()
        };
        let opts2: ScanOptions = (&args2).into();
        assert_eq!(opts2.config, ConfigMode::Auto);
    }
}

// =========================================================================
// Scenario: UserConfig profile lookup
// =========================================================================

mod user_config_profiles {
    use super::*;

    #[test]
    fn lookup_existing_profile() {
        // Given a UserConfig with a profile
        let mut profiles = BTreeMap::new();
        profiles.insert(
            "llm_safe".to_string(),
            Profile {
                format: Some("json".to_string()),
                redact: Some(tokmd_config::RedactMode::All),
                top: Some(10),
                ..Default::default()
            },
        );
        let config = UserConfig {
            profiles,
            repos: BTreeMap::new(),
        };

        // When looking up the profile
        let profile = config.profiles.get("llm_safe");

        // Then it exists with correct values
        assert!(profile.is_some());
        let p = profile.unwrap();
        assert_eq!(p.format, Some("json".to_string()));
        assert_eq!(p.top, Some(10));
    }

    #[test]
    fn lookup_missing_profile_returns_none() {
        let config = UserConfig::default();
        assert!(config.profiles.get("nonexistent").is_none());
    }

    #[test]
    fn repo_to_profile_mapping() {
        // Given a UserConfig with repo mapping
        let mut repos = BTreeMap::new();
        repos.insert("myorg/myrepo".to_string(), "ci".to_string());
        repos.insert("myorg/secret".to_string(), "llm_safe".to_string());
        let config = UserConfig {
            profiles: BTreeMap::new(),
            repos,
        };

        // When looking up repos
        assert_eq!(config.repos.get("myorg/myrepo"), Some(&"ci".to_string()));
        assert_eq!(
            config.repos.get("myorg/secret"),
            Some(&"llm_safe".to_string())
        );
        assert_eq!(config.repos.get("other/repo"), None);
    }

    #[test]
    fn multiple_profiles_are_independent() {
        let mut profiles = BTreeMap::new();
        profiles.insert(
            "a".to_string(),
            Profile {
                format: Some("json".to_string()),
                top: Some(5),
                ..Default::default()
            },
        );
        profiles.insert(
            "b".to_string(),
            Profile {
                format: Some("tsv".to_string()),
                top: Some(50),
                files: Some(true),
                ..Default::default()
            },
        );

        let config = UserConfig {
            profiles,
            repos: BTreeMap::new(),
        };

        let a = config.profiles.get("a").unwrap();
        let b = config.profiles.get("b").unwrap();

        assert_eq!(a.format, Some("json".to_string()));
        assert_eq!(b.format, Some("tsv".to_string()));
        assert_eq!(a.files, None);
        assert_eq!(b.files, Some(true));
    }
}

// =========================================================================
// Scenario: Profile defaults and fields
// =========================================================================

mod profile_defaults {
    use super::*;

    #[test]
    fn default_profile_has_all_none() {
        let p = Profile::default();
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
    }

    #[test]
    fn default_view_profile_has_all_none() {
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
        assert_eq!(p.window, None);
        assert_eq!(p.budget, None);
        assert_eq!(p.strategy, None);
        assert_eq!(p.rank_by, None);
        assert_eq!(p.output, None);
        assert_eq!(p.compress, None);
        assert_eq!(p.metric, None);
    }

    #[test]
    fn default_cli_lang_args_has_none_paths() {
        let args = CliLangArgs::default();
        assert!(args.paths.is_none());
        assert!(args.format.is_none());
        assert!(args.top.is_none());
        assert!(!args.files);
        assert!(args.children.is_none());
    }
}

// =========================================================================
// Scenario: Profile merging logic (manual Option-based overlay)
// =========================================================================

mod profile_merging {
    use super::*;

    /// Simulates how CLI code would merge a ViewProfile onto command defaults:
    /// profile fields override only when Some.
    fn merge_view_into_export(
        base_format: Option<String>,
        base_min_code: Option<usize>,
        profile: &ViewProfile,
    ) -> (Option<String>, Option<usize>) {
        let format = profile.format.clone().or(base_format);
        let min_code = profile.min_code.or(base_min_code);
        (format, min_code)
    }

    #[test]
    fn profile_overrides_base_when_some() {
        let profile = ViewProfile {
            format: Some("json".to_string()),
            min_code: Some(42),
            ..Default::default()
        };

        let (fmt, mc) = merge_view_into_export(Some("md".to_string()), Some(0), &profile);

        assert_eq!(fmt, Some("json".to_string()));
        assert_eq!(mc, Some(42));
    }

    #[test]
    fn profile_none_preserves_base() {
        let profile = ViewProfile::default(); // all None

        let (fmt, mc) = merge_view_into_export(Some("tsv".to_string()), Some(10), &profile);

        assert_eq!(fmt, Some("tsv".to_string()));
        assert_eq!(mc, Some(10));
    }

    #[test]
    fn profile_partially_overrides() {
        let profile = ViewProfile {
            format: Some("csv".to_string()),
            // min_code is None
            ..Default::default()
        };

        let (fmt, mc) = merge_view_into_export(Some("md".to_string()), Some(5), &profile);

        assert_eq!(fmt, Some("csv".to_string()));
        assert_eq!(mc, Some(5)); // base preserved
    }

    #[test]
    fn profile_overrides_none_base() {
        let profile = ViewProfile {
            format: Some("json".to_string()),
            ..Default::default()
        };

        let (fmt, mc) = merge_view_into_export(None, None, &profile);

        assert_eq!(fmt, Some("json".to_string()));
        assert_eq!(mc, None);
    }

    #[test]
    fn both_none_yields_none() {
        let profile = ViewProfile::default();
        let (fmt, mc) = merge_view_into_export(None, None, &profile);
        assert_eq!(fmt, None);
        assert_eq!(mc, None);
    }
}

// =========================================================================
// Scenario: TOML → ViewProfile round-trip through profiles
// =========================================================================

mod toml_view_roundtrip {
    use super::*;

    #[test]
    fn profile_loaded_from_toml_can_be_used_for_merging() {
        // Given a TOML config with a view profile
        let toml_str = r#"
[view.custom]
format = "json"
top = 15
redact = "all"
compress = true
"#;
        let config = TomlConfig::parse(toml_str).expect("valid");
        let profile = config.view.get("custom").expect("profile exists");

        // When using profile values in a merge
        let effective_format = profile.format.clone().or(Some("md".to_string()));
        let effective_top = profile.top.or(Some(0));
        let effective_compress = profile.compress.unwrap_or(false);

        // Then profile values win
        assert_eq!(effective_format, Some("json".to_string()));
        assert_eq!(effective_top, Some(15));
        assert!(effective_compress);
    }

    #[test]
    fn missing_profile_name_yields_none() {
        let config = TomlConfig::parse("").expect("valid");
        assert!(config.view.get("nonexistent").is_none());
    }
}

// =========================================================================
// Scenario: Clap CLI parsing via try_parse_from
// =========================================================================

mod cli_parsing {
    use clap::Parser;
    use tokmd_config::Cli;

    #[test]
    fn bare_invocation_has_no_subcommand() {
        let cli = Cli::try_parse_from(["tokmd"]).expect("parse");
        assert!(cli.command.is_none());
        assert!(cli.profile.is_none());
    }

    #[test]
    fn lang_subcommand_with_format() {
        let cli = Cli::try_parse_from(["tokmd", "lang", "--format", "json"]).expect("parse");
        match cli.command {
            Some(tokmd_config::Commands::Lang(args)) => {
                assert_eq!(args.format, Some(tokmd_config::TableFormat::Json));
            }
            other => panic!("expected Lang, got {:?}", other),
        }
    }

    #[test]
    fn module_subcommand_with_roots_and_depth() {
        let cli = Cli::try_parse_from([
            "tokmd",
            "module",
            "--module-roots",
            "src,lib",
            "--module-depth",
            "3",
        ])
        .expect("parse");
        match cli.command {
            Some(tokmd_config::Commands::Module(args)) => {
                assert_eq!(
                    args.module_roots,
                    Some(vec!["src".to_string(), "lib".to_string()])
                );
                assert_eq!(args.module_depth, Some(3));
            }
            other => panic!("expected Module, got {:?}", other),
        }
    }

    #[test]
    fn export_subcommand_with_all_options() {
        let cli = Cli::try_parse_from([
            "tokmd",
            "export",
            "--format",
            "csv",
            "--min-code",
            "5",
            "--max-rows",
            "100",
            "--redact",
            "paths",
        ])
        .expect("parse");
        match cli.command {
            Some(tokmd_config::Commands::Export(args)) => {
                assert_eq!(args.format, Some(tokmd_config::ExportFormat::Csv));
                assert_eq!(args.min_code, Some(5));
                assert_eq!(args.max_rows, Some(100));
                assert_eq!(args.redact, Some(tokmd_config::RedactMode::Paths));
            }
            other => panic!("expected Export, got {:?}", other),
        }
    }

    #[test]
    fn global_exclude_flag_is_repeatable() {
        let cli = Cli::try_parse_from(["tokmd", "--exclude", "target", "--exclude", "*.min.js"])
            .expect("parse");
        assert_eq!(
            cli.global.excluded,
            vec!["target".to_string(), "*.min.js".to_string()]
        );
    }

    #[test]
    fn global_hidden_and_no_ignore_flags() {
        let cli = Cli::try_parse_from(["tokmd", "--hidden", "--no-ignore"]).expect("parse");
        assert!(cli.global.hidden);
        assert!(cli.global.no_ignore);
    }

    #[test]
    fn profile_flag_is_parsed() {
        let cli = Cli::try_parse_from(["tokmd", "--profile", "llm_safe"]).expect("parse");
        assert_eq!(cli.profile, Some("llm_safe".to_string()));
    }

    #[test]
    fn view_alias_works_for_profile() {
        let cli = Cli::try_parse_from(["tokmd", "--view", "ci"]).expect("parse");
        assert_eq!(cli.profile, Some("ci".to_string()));
    }

    #[test]
    fn verbose_flag_counts() {
        let cli = Cli::try_parse_from(["tokmd", "-v"]).expect("parse");
        assert_eq!(cli.global.verbose, 1);

        let cli2 = Cli::try_parse_from(["tokmd", "-vvv"]).expect("parse");
        assert_eq!(cli2.global.verbose, 3);
    }

    #[test]
    fn analyze_subcommand_with_preset_and_window() {
        let cli =
            Cli::try_parse_from(["tokmd", "analyze", "--preset", "risk", "--window", "200000"])
                .expect("parse");
        match cli.command {
            Some(tokmd_config::Commands::Analyze(args)) => {
                assert_eq!(args.preset, Some(tokmd_config::AnalysisPreset::Risk));
                assert_eq!(args.window, Some(200_000));
            }
            other => panic!("expected Analyze, got {:?}", other),
        }
    }

    #[test]
    fn context_subcommand_defaults() {
        let cli = Cli::try_parse_from(["tokmd", "context"]).expect("parse");
        match cli.command {
            Some(tokmd_config::Commands::Context(args)) => {
                assert_eq!(args.budget, "128k");
                assert_eq!(args.max_file_pct, 0.15);
                assert!(!args.compress);
                assert!(!args.no_smart_exclude);
            }
            other => panic!("expected Context, got {:?}", other),
        }
    }

    #[test]
    fn diff_subcommand_with_refs() {
        let cli = Cli::try_parse_from(["tokmd", "diff", "v1.0", "v2.0"]).expect("parse");
        match cli.command {
            Some(tokmd_config::Commands::Diff(args)) => {
                assert_eq!(args.refs, vec!["v1.0".to_string(), "v2.0".to_string()]);
                assert_eq!(args.format, tokmd_config::DiffFormat::Md); // default
            }
            other => panic!("expected Diff, got {:?}", other),
        }
    }

    #[test]
    fn gate_subcommand_fail_fast() {
        let cli = Cli::try_parse_from(["tokmd", "gate", "--fail-fast"]).expect("parse");
        match cli.command {
            Some(tokmd_config::Commands::Gate(args)) => {
                assert!(args.fail_fast);
                assert_eq!(args.format, tokmd_config::GateFormat::Text);
            }
            other => panic!("expected Gate, got {:?}", other),
        }
    }

    #[test]
    fn cockpit_subcommand_defaults() {
        let cli = Cli::try_parse_from(["tokmd", "cockpit"]).expect("parse");
        match cli.command {
            Some(tokmd_config::Commands::Cockpit(args)) => {
                assert_eq!(args.base, "main");
                assert_eq!(args.head, "HEAD");
                assert_eq!(args.format, tokmd_config::CockpitFormat::Json);
                assert!(!args.sensor_mode);
            }
            other => panic!("expected Cockpit, got {:?}", other),
        }
    }

    #[test]
    fn handoff_subcommand_defaults() {
        let cli = Cli::try_parse_from(["tokmd", "handoff"]).expect("parse");
        match cli.command {
            Some(tokmd_config::Commands::Handoff(args)) => {
                assert_eq!(args.budget, "128k");
                assert_eq!(args.preset, tokmd_config::HandoffPreset::Risk);
                assert_eq!(args.rank_by, tokmd_config::ValueMetric::Hotspot);
                assert!(!args.force);
                assert!(!args.compress);
            }
            other => panic!("expected Handoff, got {:?}", other),
        }
    }

    #[test]
    fn sensor_subcommand_defaults() {
        let cli = Cli::try_parse_from(["tokmd", "sensor"]).expect("parse");
        match cli.command {
            Some(tokmd_config::Commands::Sensor(args)) => {
                assert_eq!(args.base, "main");
                assert_eq!(args.head, "HEAD");
                assert_eq!(args.format, tokmd_config::SensorFormat::Json);
            }
            other => panic!("expected Sensor, got {:?}", other),
        }
    }

    #[test]
    fn invalid_flag_fails() {
        let result = Cli::try_parse_from(["tokmd", "--not-a-real-flag"]);
        assert!(result.is_err());
    }

    #[test]
    fn no_progress_flag() {
        let cli = Cli::try_parse_from(["tokmd", "--no-progress"]).expect("parse");
        assert!(cli.global.no_progress);
    }

    #[test]
    fn config_mode_none_flag() {
        let cli = Cli::try_parse_from(["tokmd", "--config", "none"]).expect("parse");
        assert_eq!(cli.global.config, tokmd_config::ConfigMode::None);
    }
}

// =========================================================================
// Scenario: UserConfig JSON serialization
// =========================================================================

mod user_config_serialization {
    use super::*;

    #[test]
    fn empty_user_config_serializes_to_empty_maps() {
        let config = UserConfig::default();
        let json = serde_json::to_string(&config).expect("serialize");
        assert!(json.contains("\"profiles\":{}"));
        assert!(json.contains("\"repos\":{}"));
    }

    #[test]
    fn user_config_roundtrips_through_json() {
        let mut profiles = BTreeMap::new();
        profiles.insert(
            "ci".to_string(),
            Profile {
                format: Some("tsv".to_string()),
                top: Some(20),
                min_code: Some(5),
                ..Default::default()
            },
        );
        let mut repos = BTreeMap::new();
        repos.insert("org/repo".to_string(), "ci".to_string());

        let config = UserConfig { profiles, repos };
        let json = serde_json::to_string(&config).expect("serialize");
        let config2: UserConfig = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(config2.profiles.len(), 1);
        let ci = config2.profiles.get("ci").unwrap();
        assert_eq!(ci.format, Some("tsv".to_string()));
        assert_eq!(ci.top, Some(20));
        assert_eq!(config2.repos.get("org/repo"), Some(&"ci".to_string()));
    }

    #[test]
    fn profile_with_module_roots_roundtrips() {
        let mut profiles = BTreeMap::new();
        profiles.insert(
            "mono".to_string(),
            Profile {
                module_roots: Some(vec!["crates".to_string(), "packages".to_string()]),
                module_depth: Some(3),
                ..Default::default()
            },
        );
        let config = UserConfig {
            profiles,
            repos: BTreeMap::new(),
        };

        let json = serde_json::to_string(&config).expect("serialize");
        let config2: UserConfig = serde_json::from_str(&json).expect("deserialize");
        let p = config2.profiles.get("mono").unwrap();
        assert_eq!(
            p.module_roots,
            Some(vec!["crates".to_string(), "packages".to_string()])
        );
        assert_eq!(p.module_depth, Some(3));
    }
}

// =========================================================================
// Scenario: Edge cases for TOML type mismatches
// =========================================================================

mod toml_type_errors {
    use tokmd_config::TomlConfig;

    #[test]
    fn string_where_bool_expected_fails() {
        let toml_str = r#"
[scan]
hidden = "yes"
"#;
        let result = TomlConfig::parse(toml_str);
        assert!(result.is_err());
    }

    #[test]
    fn bool_where_usize_expected_fails() {
        let toml_str = r#"
[module]
depth = true
"#;
        let result = TomlConfig::parse(toml_str);
        assert!(result.is_err());
    }

    #[test]
    fn negative_number_where_usize_expected_fails() {
        let toml_str = r#"
[export]
min_code = -1
"#;
        let result = TomlConfig::parse(toml_str);
        assert!(result.is_err());
    }

    #[test]
    fn float_where_usize_expected_fails() {
        let toml_str = r#"
[analyze]
window = 1.5
"#;
        let result = TomlConfig::parse(toml_str);
        assert!(result.is_err());
    }

    #[test]
    fn array_where_string_expected_fails() {
        let toml_str = r#"
[context]
budget = ["128k"]
"#;
        let result = TomlConfig::parse(toml_str);
        assert!(result.is_err());
    }

    #[test]
    fn string_where_array_expected_fails() {
        let toml_str = r#"
[module]
roots = "crates"
"#;
        let result = TomlConfig::parse(toml_str);
        assert!(result.is_err());
    }
}
