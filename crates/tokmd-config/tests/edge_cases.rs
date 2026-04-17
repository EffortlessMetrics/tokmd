//! Edge-case and coverage-gap tests for tokmd-config.
//!
//! Focuses on areas not covered by existing test files:
//! - CLI parsing for newer subcommands (Handoff, Sensor, Baseline, Gate, Cockpit)
//! - Serde roundtrip for newer enums added after properties.rs
//! - TomlConfig with gate allow_missing fields
//! - ViewProfile with full field coverage

use tokmd_config::{
    AnalysisPreset, Cli, CockpitFormat, ColorMode, ContextOutput, ContextStrategy, DiffFormat,
    DiffRangeMode, GateFormat, HandoffPreset, NearDupScope, SensorFormat, TomlConfig, ValueMetric,
    ViewProfile,
};

// =========================================================================
// Scenario: Serde roundtrip for newer enums not covered in properties.rs
// =========================================================================

mod newer_enum_roundtrips {
    use super::*;

    #[test]
    fn context_strategy_roundtrip() {
        for variant in [ContextStrategy::Greedy, ContextStrategy::Spread] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: ContextStrategy = serde_json::from_str(&json).unwrap();
            assert_eq!(
                back, variant,
                "ContextStrategy roundtrip failed for {:?}",
                variant
            );
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
            assert_eq!(
                back, variant,
                "ValueMetric roundtrip failed for {:?}",
                variant
            );
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
            assert_eq!(
                back, variant,
                "ContextOutput roundtrip failed for {:?}",
                variant
            );
        }
    }

    #[test]
    fn gate_format_roundtrip() {
        for variant in [GateFormat::Text, GateFormat::Json] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: GateFormat = serde_json::from_str(&json).unwrap();
            assert_eq!(
                back, variant,
                "GateFormat roundtrip failed for {:?}",
                variant
            );
        }
    }

    #[test]
    fn cockpit_format_roundtrip() {
        for variant in [
            CockpitFormat::Json,
            CockpitFormat::Md,
            CockpitFormat::Sections,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: CockpitFormat = serde_json::from_str(&json).unwrap();
            assert_eq!(
                back, variant,
                "CockpitFormat roundtrip failed for {:?}",
                variant
            );
        }
    }

    #[test]
    fn handoff_preset_roundtrip() {
        for variant in [
            HandoffPreset::Minimal,
            HandoffPreset::Standard,
            HandoffPreset::Risk,
            HandoffPreset::Deep,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: HandoffPreset = serde_json::from_str(&json).unwrap();
            assert_eq!(
                back, variant,
                "HandoffPreset roundtrip failed for {:?}",
                variant
            );
        }
    }

    #[test]
    fn sensor_format_roundtrip() {
        for variant in [SensorFormat::Json, SensorFormat::Md] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: SensorFormat = serde_json::from_str(&json).unwrap();
            assert_eq!(
                back, variant,
                "SensorFormat roundtrip failed for {:?}",
                variant
            );
        }
    }

    #[test]
    fn near_dup_scope_roundtrip() {
        for variant in [
            NearDupScope::Module,
            NearDupScope::Lang,
            NearDupScope::Global,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: NearDupScope = serde_json::from_str(&json).unwrap();
            assert_eq!(
                back, variant,
                "NearDupScope roundtrip failed for {:?}",
                variant
            );
        }
    }

    #[test]
    fn diff_range_mode_roundtrip() {
        for variant in [DiffRangeMode::TwoDot, DiffRangeMode::ThreeDot] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: DiffRangeMode = serde_json::from_str(&json).unwrap();
            assert_eq!(
                back, variant,
                "DiffRangeMode roundtrip failed for {:?}",
                variant
            );
        }
    }

    #[test]
    fn diff_format_roundtrip() {
        for variant in [DiffFormat::Md, DiffFormat::Json] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: DiffFormat = serde_json::from_str(&json).unwrap();
            assert_eq!(
                back, variant,
                "DiffFormat roundtrip failed for {:?}",
                variant
            );
        }
    }

    #[test]
    fn color_mode_roundtrip() {
        for variant in [ColorMode::Auto, ColorMode::Always, ColorMode::Never] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: ColorMode = serde_json::from_str(&json).unwrap();
            assert_eq!(
                back, variant,
                "ColorMode roundtrip failed for {:?}",
                variant
            );
        }
    }

    #[test]
    fn newer_enums_use_kebab_case() {
        // All newer enums must serialize to kebab-case (no underscores, no uppercase)
        let cases: Vec<String> = vec![
            serde_json::to_string(&ContextStrategy::Greedy).unwrap(),
            serde_json::to_string(&ValueMetric::Hotspot).unwrap(),
            serde_json::to_string(&ContextOutput::List).unwrap(),
            serde_json::to_string(&GateFormat::Text).unwrap(),
            serde_json::to_string(&CockpitFormat::Sections).unwrap(),
            serde_json::to_string(&HandoffPreset::Minimal).unwrap(),
            serde_json::to_string(&SensorFormat::Json).unwrap(),
            serde_json::to_string(&NearDupScope::Global).unwrap(),
            serde_json::to_string(&DiffRangeMode::TwoDot).unwrap(),
            serde_json::to_string(&DiffFormat::Md).unwrap(),
            serde_json::to_string(&ColorMode::Always).unwrap(),
        ];

        for json in &cases {
            let inner = json.trim_matches('"');
            assert!(
                !inner.contains('_') || inner.contains('-'),
                "Expected kebab-case (no bare underscores), got: {}",
                inner
            );
            assert!(
                !inner.chars().any(|c| c.is_uppercase()),
                "Expected lowercase, got: {}",
                inner
            );
        }
    }
}

// =========================================================================
// Scenario: CLI parsing for newer subcommands via try_parse_from
// =========================================================================

mod cli_parsing {
    use super::*;
    use clap::Parser;

    #[test]
    fn handoff_subcommand_parses_with_defaults() {
        let cli = Cli::try_parse_from(["tokmd", "handoff"]).unwrap();
        let cmd = cli.command.expect("should have subcommand");
        match cmd {
            tokmd_config::Commands::Handoff(args) => {
                assert_eq!(args.budget, "128k");
                assert!(!args.force);
                assert!(!args.compress);
                assert!(!args.no_smart_exclude);
                assert!(!args.no_git);
            }
            other => panic!("Expected Handoff, got {:?}", other),
        }
    }

    #[test]
    fn handoff_subcommand_with_custom_flags() {
        let cli = Cli::try_parse_from([
            "tokmd",
            "handoff",
            "--budget",
            "256k",
            "--preset",
            "deep",
            "--force",
            "--compress",
            "--no-smart-exclude",
            "--no-git",
        ])
        .unwrap();

        match cli.command.unwrap() {
            tokmd_config::Commands::Handoff(args) => {
                assert_eq!(args.budget, "256k");
                assert_eq!(args.preset, HandoffPreset::Deep);
                assert!(args.force);
                assert!(args.compress);
                assert!(args.no_smart_exclude);
                assert!(args.no_git);
            }
            other => panic!("Expected Handoff, got {:?}", other),
        }
    }

    #[test]
    fn sensor_subcommand_parses_with_defaults() {
        let cli = Cli::try_parse_from(["tokmd", "sensor"]).unwrap();
        match cli.command.unwrap() {
            tokmd_config::Commands::Sensor(args) => {
                assert_eq!(args.base, "main");
                assert_eq!(args.head, "HEAD");
                assert_eq!(args.format, SensorFormat::Json);
            }
            other => panic!("Expected Sensor, got {:?}", other),
        }
    }

    #[test]
    fn sensor_subcommand_with_custom_refs() {
        let cli = Cli::try_parse_from([
            "tokmd",
            "sensor",
            "--base",
            "v1.0",
            "--head",
            "feature-branch",
            "--format",
            "md",
        ])
        .unwrap();

        match cli.command.unwrap() {
            tokmd_config::Commands::Sensor(args) => {
                assert_eq!(args.base, "v1.0");
                assert_eq!(args.head, "feature-branch");
                assert_eq!(args.format, SensorFormat::Md);
            }
            other => panic!("Expected Sensor, got {:?}", other),
        }
    }

    #[test]
    fn baseline_subcommand_parses_with_defaults() {
        let cli = Cli::try_parse_from(["tokmd", "baseline"]).unwrap();
        match cli.command.unwrap() {
            tokmd_config::Commands::Baseline(args) => {
                assert_eq!(args.path.to_str().unwrap(), ".");
                assert_eq!(args.output.to_str().unwrap(), ".tokmd/baseline.json");
                assert!(!args.determinism);
                assert!(!args.force);
            }
            other => panic!("Expected Baseline, got {:?}", other),
        }
    }

    #[test]
    fn gate_subcommand_parses_with_defaults() {
        let cli = Cli::try_parse_from(["tokmd", "gate"]).unwrap();
        match cli.command.unwrap() {
            tokmd_config::Commands::Gate(args) => {
                assert!(args.input.is_none());
                assert!(args.policy.is_none());
                assert!(args.baseline.is_none());
                assert!(args.ratchet_config.is_none());
                assert!(args.preset.is_none());
                assert_eq!(args.format, GateFormat::Text);
                assert!(!args.fail_fast);
            }
            other => panic!("Expected Gate, got {:?}", other),
        }
    }

    #[test]
    fn gate_subcommand_with_all_options() {
        let cli = Cli::try_parse_from([
            "tokmd",
            "gate",
            "receipt.json",
            "--policy",
            "policy.toml",
            "--baseline",
            "baseline.json",
            "--ratchet-config",
            "ratchet.toml",
            "--preset",
            "health",
            "--format",
            "json",
            "--fail-fast",
        ])
        .unwrap();

        match cli.command.unwrap() {
            tokmd_config::Commands::Gate(args) => {
                assert_eq!(args.input.unwrap().to_str().unwrap(), "receipt.json");
                assert_eq!(args.policy.unwrap().to_str().unwrap(), "policy.toml");
                assert_eq!(args.baseline.unwrap().to_str().unwrap(), "baseline.json");
                assert_eq!(
                    args.ratchet_config.unwrap().to_str().unwrap(),
                    "ratchet.toml"
                );
                assert_eq!(args.preset, Some(AnalysisPreset::Health));
                assert_eq!(args.format, GateFormat::Json);
                assert!(args.fail_fast);
            }
            other => panic!("Expected Gate, got {:?}", other),
        }
    }

    #[test]
    fn cockpit_subcommand_with_sensor_mode() {
        let cli = Cli::try_parse_from([
            "tokmd",
            "cockpit",
            "--base",
            "v2.0",
            "--head",
            "feature",
            "--diff-range",
            "three-dot",
            "--sensor-mode",
        ])
        .unwrap();

        match cli.command.unwrap() {
            tokmd_config::Commands::Cockpit(args) => {
                assert_eq!(args.base, "v2.0");
                assert_eq!(args.head, "feature");
                assert_eq!(args.diff_range, DiffRangeMode::ThreeDot);
                assert!(args.sensor_mode);
            }
            other => panic!("Expected Cockpit, got {:?}", other),
        }
    }

    #[test]
    fn context_subcommand_with_advanced_options() {
        let cli = Cli::try_parse_from([
            "tokmd",
            "context",
            "--budget",
            "1m",
            "--strategy",
            "spread",
            "--rank-by",
            "churn",
            "--mode",
            "bundle",
            "--compress",
            "--no-smart-exclude",
            "--max-file-pct",
            "0.25",
            "--require-git-scores",
        ])
        .unwrap();

        match cli.command.unwrap() {
            tokmd_config::Commands::Context(args) => {
                assert_eq!(args.budget, "1m");
                assert_eq!(args.strategy, ContextStrategy::Spread);
                assert_eq!(args.rank_by, ValueMetric::Churn);
                assert_eq!(args.output_mode, ContextOutput::Bundle);
                assert!(args.compress);
                assert!(args.no_smart_exclude);
                assert!((args.max_file_pct - 0.25).abs() < 1e-10);
                assert!(args.require_git_scores);
            }
            other => panic!("Expected Context, got {:?}", other),
        }
    }

    #[test]
    fn analyze_subcommand_with_near_dup_options() {
        let cli = Cli::try_parse_from([
            "tokmd",
            "analyze",
            "--near-dup",
            "--near-dup-threshold",
            "0.90",
            "--near-dup-max-files",
            "5000",
            "--near-dup-scope",
            "global",
            "--near-dup-max-pairs",
            "500",
            "--near-dup-exclude",
            "*.generated.*",
        ])
        .unwrap();

        match cli.command.unwrap() {
            tokmd_config::Commands::Analyze(args) => {
                assert!(args.near_dup);
                assert!((args.near_dup_threshold - 0.90).abs() < 1e-10);
                assert_eq!(args.near_dup_max_files, 5000);
                assert_eq!(args.near_dup_scope, Some(NearDupScope::Global));
                assert_eq!(args.near_dup_max_pairs, 500);
                assert_eq!(args.near_dup_exclude, vec!["*.generated.*"]);
            }
            other => panic!("Expected Analyze, got {:?}", other),
        }
    }
}

// =========================================================================
// Scenario: TomlConfig with gate allow_missing fields
// =========================================================================

mod toml_gate_config {
    use super::*;

    #[test]
    fn gate_config_with_allow_missing_baseline() {
        let toml_str = r#"
[gate]
allow_missing_baseline = true
allow_missing_current = false
"#;
        let config = TomlConfig::parse(toml_str).expect("valid");
        assert_eq!(config.gate.allow_missing_baseline, Some(true));
        assert_eq!(config.gate.allow_missing_current, Some(false));
    }

    #[test]
    fn gate_config_with_both_allow_missing_true() {
        let toml_str = r#"
[gate]
allow_missing_baseline = true
allow_missing_current = true
fail_fast = false
"#;
        let config = TomlConfig::parse(toml_str).expect("valid");
        assert_eq!(config.gate.allow_missing_baseline, Some(true));
        assert_eq!(config.gate.allow_missing_current, Some(true));
        assert_eq!(config.gate.fail_fast, Some(false));
    }

    #[test]
    fn gate_config_with_preset_and_policy() {
        let toml_str = r#"
[gate]
policy = "policy.toml"
baseline = "baseline.json"
preset = "health"
"#;
        let config = TomlConfig::parse(toml_str).expect("valid");
        assert_eq!(config.gate.policy, Some("policy.toml".to_string()));
        assert_eq!(config.gate.baseline, Some("baseline.json".to_string()));
        assert_eq!(config.gate.preset, Some("health".to_string()));
    }

    #[test]
    fn gate_config_defaults_are_none_when_absent() {
        let toml_str = r#"
[gate]
fail_fast = true
"#;
        let config = TomlConfig::parse(toml_str).expect("valid");
        assert_eq!(config.gate.fail_fast, Some(true));
        assert!(config.gate.allow_missing_baseline.is_none());
        assert!(config.gate.allow_missing_current.is_none());
        assert!(config.gate.policy.is_none());
        assert!(config.gate.baseline.is_none());
        assert!(config.gate.preset.is_none());
    }
}

// =========================================================================
// Scenario: ViewProfile with all fields populated
// =========================================================================

mod view_profile_coverage {
    use super::*;

    #[test]
    fn view_profile_with_all_fields_set() {
        let toml_str = r#"
[view.full]
format = "json"
top = 25
files = true
module_roots = ["crates", "packages"]
module_depth = 3
min_code = 10
max_rows = 500
redact = "all"
meta = true
children = "collapse"
preset = "deep"
window = 256000
budget = "128k"
strategy = "greedy"
rank_by = "code"
output = "list"
compress = true
metric = "tokens"
"#;
        let config = TomlConfig::parse(toml_str).expect("valid");
        let full = config.view.get("full").expect("full profile");

        assert_eq!(full.format, Some("json".to_string()));
        assert_eq!(full.top, Some(25));
        assert_eq!(full.files, Some(true));
        assert_eq!(
            full.module_roots,
            Some(vec!["crates".to_string(), "packages".to_string()])
        );
        assert_eq!(full.module_depth, Some(3));
        assert_eq!(full.min_code, Some(10));
        assert_eq!(full.max_rows, Some(500));
        assert_eq!(full.redact, Some("all".to_string()));
        assert_eq!(full.meta, Some(true));
        assert_eq!(full.children, Some("collapse".to_string()));
        assert_eq!(full.preset, Some("deep".to_string()));
        assert_eq!(full.window, Some(256000));
        assert_eq!(full.budget, Some("128k".to_string()));
        assert_eq!(full.strategy, Some("greedy".to_string()));
        assert_eq!(full.rank_by, Some("code".to_string()));
        assert_eq!(full.output, Some("list".to_string()));
        assert_eq!(full.compress, Some(true));
        assert_eq!(full.metric, Some("tokens".to_string()));
    }

    #[test]
    fn view_profile_json_roundtrip_with_all_fields() {
        let profile = ViewProfile {
            format: Some("json".to_string()),
            top: Some(10),
            files: Some(true),
            module_roots: Some(vec!["src".to_string()]),
            module_depth: Some(2),
            min_code: Some(5),
            max_rows: Some(100),
            redact: Some("paths".to_string()),
            meta: Some(false),
            children: Some("separate".to_string()),
            preset: Some("health".to_string()),
            window: Some(128000),
            budget: Some("64k".to_string()),
            strategy: Some("spread".to_string()),
            rank_by: Some("tokens".to_string()),
            output: Some("bundle".to_string()),
            compress: Some(false),
            metric: Some("lines".to_string()),
        };

        let json = serde_json::to_string(&profile).unwrap();
        let back: ViewProfile = serde_json::from_str(&json).unwrap();
        assert_eq!(back.format, profile.format);
        assert_eq!(back.top, profile.top);
        assert_eq!(back.files, profile.files);
        assert_eq!(back.module_roots, profile.module_roots);
        assert_eq!(back.module_depth, profile.module_depth);
        assert_eq!(back.min_code, profile.min_code);
        assert_eq!(back.max_rows, profile.max_rows);
        assert_eq!(back.redact, profile.redact);
        assert_eq!(back.meta, profile.meta);
        assert_eq!(back.children, profile.children);
        assert_eq!(back.preset, profile.preset);
        assert_eq!(back.window, profile.window);
        assert_eq!(back.budget, profile.budget);
        assert_eq!(back.strategy, profile.strategy);
        assert_eq!(back.rank_by, profile.rank_by);
        assert_eq!(back.output, profile.output);
        assert_eq!(back.compress, profile.compress);
        assert_eq!(back.metric, profile.metric);
    }

    #[test]
    fn view_profile_default_is_all_none() {
        let profile = ViewProfile::default();
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
        assert!(profile.preset.is_none());
        assert!(profile.window.is_none());
        assert!(profile.budget.is_none());
        assert!(profile.strategy.is_none());
        assert!(profile.rank_by.is_none());
        assert!(profile.output.is_none());
        assert!(profile.compress.is_none());
        assert!(profile.metric.is_none());
    }
}
