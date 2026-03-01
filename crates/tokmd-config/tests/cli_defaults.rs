//! Tests verifying CLI default values match documented expectations.
//!
//! These tests parse subcommands with minimal args and assert that every
//! default flag / value is exactly what the help text promises.

use clap::Parser;
use tokmd_config::{
    Cli, CockpitFormat, ColorMode, ContextOutput, ContextStrategy, DiffFormat, DiffRangeMode,
    GateFormat, HandoffPreset, NearDupScope, SensorFormat, ValueMetric,
};

// =========================================================================
// Enum default values (via Default trait)
// =========================================================================

#[test]
fn diff_format_default_is_md() {
    assert_eq!(DiffFormat::default(), DiffFormat::Md);
}

#[test]
fn color_mode_default_is_auto() {
    assert_eq!(ColorMode::default(), ColorMode::Auto);
}

#[test]
fn cockpit_format_default_is_json() {
    assert_eq!(CockpitFormat::default(), CockpitFormat::Json);
}

#[test]
fn gate_format_default_is_text() {
    assert_eq!(GateFormat::default(), GateFormat::Text);
}

#[test]
fn context_strategy_default_is_greedy() {
    assert_eq!(ContextStrategy::default(), ContextStrategy::Greedy);
}

#[test]
fn value_metric_default_is_code() {
    assert_eq!(ValueMetric::default(), ValueMetric::Code);
}

#[test]
fn context_output_default_is_list() {
    assert_eq!(ContextOutput::default(), ContextOutput::List);
}

#[test]
fn near_dup_scope_default_is_module() {
    assert_eq!(NearDupScope::default(), NearDupScope::Module);
}

#[test]
fn diff_range_mode_default_is_two_dot() {
    assert_eq!(DiffRangeMode::default(), DiffRangeMode::TwoDot);
}

#[test]
fn handoff_preset_default_is_risk() {
    assert_eq!(HandoffPreset::default(), HandoffPreset::Risk);
}

#[test]
fn sensor_format_default_is_json() {
    assert_eq!(SensorFormat::default(), SensorFormat::Json);
}

// =========================================================================
// Subcommand-level defaults via Clap parsing
// =========================================================================

#[test]
fn baseline_subcommand_defaults() {
    let cli = Cli::try_parse_from(["tokmd", "baseline"]).expect("parse");
    match cli.command {
        Some(tokmd_config::Commands::Baseline(args)) => {
            assert_eq!(args.path.to_str().unwrap(), ".");
            assert_eq!(args.output.to_str().unwrap(), ".tokmd/baseline.json");
            assert!(!args.determinism);
            assert!(!args.force);
        }
        other => panic!("expected Baseline, got {:?}", other),
    }
}

#[test]
fn init_subcommand_defaults() {
    let cli = Cli::try_parse_from(["tokmd", "init"]).expect("parse");
    match cli.command {
        Some(tokmd_config::Commands::Init(args)) => {
            assert_eq!(args.dir.to_str().unwrap(), ".");
            assert!(!args.force);
            assert!(!args.print);
            assert_eq!(args.template, tokmd_config::InitProfile::Default);
            assert!(!args.non_interactive);
        }
        other => panic!("expected Init, got {:?}", other),
    }
}

#[test]
fn tools_subcommand_defaults() {
    let cli = Cli::try_parse_from(["tokmd", "tools"]).expect("parse");
    match cli.command {
        Some(tokmd_config::Commands::Tools(args)) => {
            assert_eq!(args.format, tokmd_config::ToolSchemaFormat::Jsonschema);
            assert!(!args.pretty);
        }
        other => panic!("expected Tools, got {:?}", other),
    }
}

#[test]
fn run_subcommand_defaults() {
    let cli = Cli::try_parse_from(["tokmd", "run"]).expect("parse");
    match cli.command {
        Some(tokmd_config::Commands::Run(args)) => {
            assert_eq!(args.paths, vec![std::path::PathBuf::from(".")]);
            assert!(args.output_dir.is_none());
            assert!(args.name.is_none());
            assert!(args.analysis.is_none());
            assert!(args.redact.is_none());
        }
        other => panic!("expected Run, got {:?}", other),
    }
}

#[test]
fn diff_subcommand_defaults() {
    let cli = Cli::try_parse_from(["tokmd", "diff"]).expect("parse");
    match cli.command {
        Some(tokmd_config::Commands::Diff(args)) => {
            assert!(args.from.is_none());
            assert!(args.to.is_none());
            assert!(args.refs.is_empty());
            assert_eq!(args.format, DiffFormat::Md);
            assert!(!args.compact);
            assert_eq!(args.color, ColorMode::Auto);
        }
        other => panic!("expected Diff, got {:?}", other),
    }
}

#[test]
fn analyze_subcommand_near_dup_defaults() {
    let cli = Cli::try_parse_from(["tokmd", "analyze"]).expect("parse");
    match cli.command {
        Some(tokmd_config::Commands::Analyze(args)) => {
            assert!(!args.near_dup);
            assert!((args.near_dup_threshold - 0.80).abs() < f64::EPSILON);
            assert_eq!(args.near_dup_max_files, 2000);
            assert_eq!(args.near_dup_max_pairs, 10000);
            assert!(args.near_dup_scope.is_none());
            assert!(args.near_dup_exclude.is_empty());
            assert!(!args.detail_functions);
            assert!(args.explain.is_none());
        }
        other => panic!("expected Analyze, got {:?}", other),
    }
}

#[test]
fn context_subcommand_git_defaults() {
    let cli = Cli::try_parse_from(["tokmd", "context"]).expect("parse");
    match cli.command {
        Some(tokmd_config::Commands::Context(args)) => {
            assert!(!args.git);
            assert!(!args.no_git);
            assert_eq!(args.max_commits, 1000);
            assert_eq!(args.max_commit_files, 100);
            assert!(args.output.is_none());
            assert!(!args.force);
            assert!(args.bundle_dir.is_none());
            assert_eq!(args.max_output_bytes, 10_485_760);
            assert!(args.log.is_none());
            assert!(args.max_file_tokens.is_none());
            assert!(!args.require_git_scores);
            assert_eq!(args.strategy, ContextStrategy::Greedy);
            assert_eq!(args.rank_by, ValueMetric::Code);
            assert_eq!(args.output_mode, ContextOutput::List);
        }
        other => panic!("expected Context, got {:?}", other),
    }
}

#[test]
fn cockpit_subcommand_sensor_and_range_defaults() {
    let cli = Cli::try_parse_from(["tokmd", "cockpit"]).expect("parse");
    match cli.command {
        Some(tokmd_config::Commands::Cockpit(args)) => {
            assert!(!args.sensor_mode);
            assert_eq!(args.diff_range, DiffRangeMode::TwoDot);
            assert!(args.baseline.is_none());
            assert!(args.output.is_none());
            assert!(args.artifacts_dir.is_none());
        }
        other => panic!("expected Cockpit, got {:?}", other),
    }
}

#[test]
fn gate_subcommand_all_defaults() {
    let cli = Cli::try_parse_from(["tokmd", "gate"]).expect("parse");
    match cli.command {
        Some(tokmd_config::Commands::Gate(args)) => {
            assert!(args.input.is_none());
            assert!(args.policy.is_none());
            assert!(args.baseline.is_none());
            assert!(args.ratchet_config.is_none());
            assert!(args.preset.is_none());
            assert_eq!(args.format, GateFormat::Text);
            assert!(!args.fail_fast);
        }
        other => panic!("expected Gate, got {:?}", other),
    }
}

#[test]
fn handoff_subcommand_all_defaults() {
    let cli = Cli::try_parse_from(["tokmd", "handoff"]).expect("parse");
    match cli.command {
        Some(tokmd_config::Commands::Handoff(args)) => {
            assert!(args.paths.is_none());
            assert_eq!(args.out_dir.to_str().unwrap(), ".handoff");
            assert_eq!(args.budget, "128k");
            assert_eq!(args.strategy, ContextStrategy::Greedy);
            assert_eq!(args.rank_by, ValueMetric::Hotspot);
            assert_eq!(args.preset, HandoffPreset::Risk);
            assert!(args.module_roots.is_none());
            assert!(args.module_depth.is_none());
            assert!(!args.force);
            assert!(!args.compress);
            assert!(!args.no_smart_exclude);
            assert!(!args.no_git);
            assert_eq!(args.max_commits, 1000);
            assert_eq!(args.max_commit_files, 100);
            assert!((args.max_file_pct - 0.15).abs() < f64::EPSILON);
            assert!(args.max_file_tokens.is_none());
        }
        other => panic!("expected Handoff, got {:?}", other),
    }
}

#[test]
fn sensor_subcommand_output_path_default() {
    let cli = Cli::try_parse_from(["tokmd", "sensor"]).expect("parse");
    match cli.command {
        Some(tokmd_config::Commands::Sensor(args)) => {
            assert_eq!(args.output.to_str().unwrap(), "artifacts/tokmd/report.json");
        }
        other => panic!("expected Sensor, got {:?}", other),
    }
}
