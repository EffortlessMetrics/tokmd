//! Edge-case tests for Clap CLI parsing.
//!
//! Covers argument conflicts, multi-value parsing, alias resolution,
//! and subcommand-specific flag forwarding.

use clap::Parser;
use tokmd_config::Cli;

// =========================================================================
// Argument conflicts: --git vs --no-git
// =========================================================================

#[test]
fn analyze_git_and_no_git_conflict() {
    let result = Cli::try_parse_from(["tokmd", "analyze", "--git", "--no-git"]);
    assert!(result.is_err(), "--git and --no-git should conflict");
}

#[test]
fn badge_git_and_no_git_conflict() {
    let result = Cli::try_parse_from(["tokmd", "badge", "--metric", "lines", "--git", "--no-git"]);
    assert!(result.is_err(), "--git and --no-git should conflict");
}

// =========================================================================
// Multi-value / repeatable arguments
// =========================================================================

#[test]
fn analyze_multiple_inputs() {
    let cli =
        Cli::try_parse_from(["tokmd", "analyze", "dir1", "dir2", "receipt.json"]).expect("parse");
    match cli.command {
        Some(tokmd_config::Commands::Analyze(args)) => {
            assert_eq!(args.inputs.len(), 3);
        }
        other => panic!("expected Analyze, got {:?}", other),
    }
}

#[test]
fn analyze_near_dup_exclude_repeatable() {
    let cli = Cli::try_parse_from([
        "tokmd",
        "analyze",
        "--near-dup",
        "--near-dup-exclude",
        "*.lock",
        "--near-dup-exclude",
        "vendor/**",
    ])
    .expect("parse");
    match cli.command {
        Some(tokmd_config::Commands::Analyze(args)) => {
            assert!(args.near_dup);
            assert_eq!(args.near_dup_exclude, vec!["*.lock", "vendor/**"]);
        }
        other => panic!("expected Analyze, got {:?}", other),
    }
}

#[test]
fn export_output_alias_works() {
    let cli = Cli::try_parse_from(["tokmd", "export", "--out", "data.jsonl"]).expect("parse");
    match cli.command {
        Some(tokmd_config::Commands::Export(args)) => {
            assert_eq!(args.output.unwrap().to_str().unwrap(), "data.jsonl");
        }
        other => panic!("expected Export, got {:?}", other),
    }
}

#[test]
fn context_output_alias_works() {
    let cli = Cli::try_parse_from(["tokmd", "context", "--out", "ctx.json"]).expect("parse");
    match cli.command {
        Some(tokmd_config::Commands::Context(args)) => {
            assert_eq!(args.output.unwrap().to_str().unwrap(), "ctx.json");
        }
        other => panic!("expected Context, got {:?}", other),
    }
}

#[test]
fn global_ignore_alias_works() {
    let cli = Cli::try_parse_from(["tokmd", "--ignore", "target"]).expect("parse");
    assert_eq!(cli.global.excluded, vec!["target".to_string()]);
}

#[test]
fn global_no_ignore_git_alias_works() {
    let cli = Cli::try_parse_from(["tokmd", "--no-ignore-git"]).expect("parse");
    assert!(cli.global.no_ignore_vcs);
}

// =========================================================================
// Subcommand-specific flag values
// =========================================================================

#[test]
fn analyze_explain_flag() {
    let cli =
        Cli::try_parse_from(["tokmd", "analyze", "--explain", "todo_density"]).expect("parse");
    match cli.command {
        Some(tokmd_config::Commands::Analyze(args)) => {
            assert_eq!(args.explain, Some("todo_density".to_string()));
        }
        other => panic!("expected Analyze, got {:?}", other),
    }
}

#[test]
fn analyze_near_dup_threshold_custom() {
    let cli = Cli::try_parse_from([
        "tokmd",
        "analyze",
        "--near-dup",
        "--near-dup-threshold",
        "0.95",
        "--near-dup-max-files",
        "500",
        "--near-dup-max-pairs",
        "1000",
        "--near-dup-scope",
        "global",
    ])
    .expect("parse");
    match cli.command {
        Some(tokmd_config::Commands::Analyze(args)) => {
            assert!((args.near_dup_threshold - 0.95).abs() < f64::EPSILON);
            assert_eq!(args.near_dup_max_files, 500);
            assert_eq!(args.near_dup_max_pairs, 1000);
            assert_eq!(
                args.near_dup_scope,
                Some(tokmd_config::NearDupScope::Global)
            );
        }
        other => panic!("expected Analyze, got {:?}", other),
    }
}

#[test]
fn diff_with_from_to_flags() {
    let cli = Cli::try_parse_from([
        "tokmd",
        "diff",
        "--from",
        "v1.0",
        "--to",
        "v2.0",
        "--format",
        "json",
        "--compact",
        "--color",
        "never",
    ])
    .expect("parse");
    match cli.command {
        Some(tokmd_config::Commands::Diff(args)) => {
            assert_eq!(args.from, Some("v1.0".to_string()));
            assert_eq!(args.to, Some("v2.0".to_string()));
            assert_eq!(args.format, tokmd_config::DiffFormat::Json);
            assert!(args.compact);
            assert_eq!(args.color, tokmd_config::ColorMode::Never);
        }
        other => panic!("expected Diff, got {:?}", other),
    }
}

#[test]
fn cockpit_with_all_flags() {
    let cli = Cli::try_parse_from([
        "tokmd",
        "cockpit",
        "--base",
        "develop",
        "--head",
        "feature/x",
        "--format",
        "md",
        "--sensor-mode",
        "--diff-range",
        "three-dot",
        "--baseline",
        "base.json",
        "--artifacts-dir",
        "out/",
    ])
    .expect("parse");
    match cli.command {
        Some(tokmd_config::Commands::Cockpit(args)) => {
            assert_eq!(args.base, "develop");
            assert_eq!(args.head, "feature/x");
            assert_eq!(args.format, tokmd_config::CockpitFormat::Md);
            assert!(args.sensor_mode);
            assert_eq!(args.diff_range, tokmd_config::DiffRangeMode::ThreeDot);
            assert_eq!(args.baseline.unwrap().to_str().unwrap(), "base.json");
            assert_eq!(args.artifacts_dir.unwrap().to_str().unwrap(), "out/");
        }
        other => panic!("expected Cockpit, got {:?}", other),
    }
}

#[test]
fn run_with_all_flags() {
    let cli = Cli::try_parse_from([
        "tokmd",
        "run",
        "src",
        "lib",
        "--output-dir",
        "artifacts",
        "--name",
        "nightly",
        "--analysis",
        "deep",
        "--redact",
        "all",
    ])
    .expect("parse");
    match cli.command {
        Some(tokmd_config::Commands::Run(args)) => {
            assert_eq!(args.paths.len(), 2);
            assert_eq!(args.output_dir.unwrap().to_str().unwrap(), "artifacts");
            assert_eq!(args.name, Some("nightly".to_string()));
            assert_eq!(args.analysis, Some(tokmd_config::AnalysisPreset::Deep));
            assert_eq!(args.redact, Some(tokmd_config::RedactMode::All));
        }
        other => panic!("expected Run, got {:?}", other),
    }
}

#[test]
fn init_with_all_flags() {
    let cli = Cli::try_parse_from([
        "tokmd",
        "init",
        "--dir",
        "/tmp/project",
        "--force",
        "--print",
        "--template",
        "rust",
        "--non-interactive",
    ])
    .expect("parse");
    match cli.command {
        Some(tokmd_config::Commands::Init(args)) => {
            assert_eq!(args.dir.to_str().unwrap(), "/tmp/project");
            assert!(args.force);
            assert!(args.print);
            assert_eq!(args.template, tokmd_config::InitProfile::Rust);
            assert!(args.non_interactive);
        }
        other => panic!("expected Init, got {:?}", other),
    }
}

#[test]
fn check_ignore_requires_paths() {
    let result = Cli::try_parse_from(["tokmd", "check-ignore"]);
    assert!(result.is_err(), "check-ignore requires at least one path");
}

#[test]
fn check_ignore_with_paths_and_verbose() {
    let cli = Cli::try_parse_from([
        "tokmd",
        "check-ignore",
        "src/main.rs",
        "target/debug/build",
        "-v",
    ])
    .expect("parse");
    match cli.command {
        Some(tokmd_config::Commands::CheckIgnore(args)) => {
            assert_eq!(args.paths.len(), 2);
            assert!(args.verbose);
        }
        other => panic!("expected CheckIgnore, got {:?}", other),
    }
}

#[test]
fn badge_subcommand_with_all_metrics() {
    let metrics = ["lines", "tokens", "bytes", "doc", "blank", "hotspot"];
    for metric in metrics {
        let cli = Cli::try_parse_from(["tokmd", "badge", "--metric", metric]).expect("parse");
        match cli.command {
            Some(tokmd_config::Commands::Badge(_)) => {}
            other => panic!("expected Badge for metric '{}', got {:?}", metric, other),
        }
    }
}

#[test]
fn completions_all_shells() {
    let shells = ["bash", "fish", "zsh", "powershell", "elvish"];
    for shell in shells {
        let cli = Cli::try_parse_from(["tokmd", "completions", shell]).expect("parse");
        match cli.command {
            Some(tokmd_config::Commands::Completions(_)) => {}
            other => panic!(
                "expected Completions for shell '{}', got {:?}",
                shell, other
            ),
        }
    }
}

#[test]
fn export_meta_explicit_true_and_false() {
    // --meta true
    let cli = Cli::try_parse_from(["tokmd", "export", "--meta", "true"]).expect("parse");
    match cli.command {
        Some(tokmd_config::Commands::Export(args)) => {
            assert_eq!(args.meta, Some(true));
        }
        other => panic!("expected Export, got {:?}", other),
    }

    // --meta false
    let cli = Cli::try_parse_from(["tokmd", "export", "--meta", "false"]).expect("parse");
    match cli.command {
        Some(tokmd_config::Commands::Export(args)) => {
            assert_eq!(args.meta, Some(false));
        }
        other => panic!("expected Export, got {:?}", other),
    }
}

#[test]
fn sensor_with_custom_flags() {
    let cli = Cli::try_parse_from([
        "tokmd", "sensor", "--base", "release", "--head", "main", "--output", "out.json",
        "--format", "md",
    ])
    .expect("parse");
    match cli.command {
        Some(tokmd_config::Commands::Sensor(args)) => {
            assert_eq!(args.base, "release");
            assert_eq!(args.head, "main");
            assert_eq!(args.output.to_str().unwrap(), "out.json");
            assert_eq!(args.format, tokmd_config::SensorFormat::Md);
        }
        other => panic!("expected Sensor, got {:?}", other),
    }
}

// =========================================================================
// Global flags interact with subcommands
// =========================================================================

#[test]
fn global_flags_with_subcommand() {
    let cli = Cli::try_parse_from([
        "tokmd",
        "--hidden",
        "--no-ignore",
        "--treat-doc-strings-as-comments",
        "--no-progress",
        "--config",
        "none",
        "lang",
        "--format",
        "json",
    ])
    .expect("parse");
    assert!(cli.global.hidden);
    assert!(cli.global.no_ignore);
    assert!(cli.global.treat_doc_strings_as_comments);
    assert!(cli.global.no_progress);
    assert_eq!(cli.global.config, tokmd_config::ConfigMode::None);
    match cli.command {
        Some(tokmd_config::Commands::Lang(args)) => {
            assert_eq!(args.format, Some(tokmd_config::TableFormat::Json));
        }
        other => panic!("expected Lang, got {:?}", other),
    }
}

#[test]
fn all_ignore_flags_set_together() {
    let cli = Cli::try_parse_from([
        "tokmd",
        "--no-ignore",
        "--no-ignore-parent",
        "--no-ignore-dot",
        "--no-ignore-vcs",
    ])
    .expect("parse");
    assert!(cli.global.no_ignore);
    assert!(cli.global.no_ignore_parent);
    assert!(cli.global.no_ignore_dot);
    assert!(cli.global.no_ignore_vcs);
}
