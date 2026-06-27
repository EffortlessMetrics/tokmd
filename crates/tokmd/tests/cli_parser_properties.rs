//! Property-based tests for CLI parser invariants.
//!
//! Verifies that the clap parser (`tokmd::cli::Cli`) never panics
//! when fed arbitrary string arguments.

use std::collections::BTreeSet;

use clap::{CommandFactory, Parser};
use proptest::prelude::*;
use tokmd::cli::Cli;

const PARSER_SUBCOMMANDS: &[&str] = &[
    "lang",
    "module",
    "export",
    "analyze",
    "badge",
    "init",
    "completions",
    "run",
    "diff",
    "context",
    "check-ignore",
    "tools",
    "gate",
    "cockpit",
    "baseline",
    "handoff",
    "sensor",
    #[cfg(feature = "ast")]
    "syntax",
    "evidence-packet",
    "packet",
    "render",
];

const ENUMS_TO_FUZZ: &[&str] = &["--config", "--format", "--children", "--redact"];

fn parser_subcommands() -> impl Strategy<Value = &'static str> {
    prop::sample::select(PARSER_SUBCOMMANDS.to_vec())
}

fn config_keys() -> impl Strategy<Value = &'static str> {
    prop::sample::select(ENUMS_TO_FUZZ.to_vec())
}

#[test]
fn parser_subcommand_property_list_matches_clap_surface() {
    let command = Cli::command();
    let actual: BTreeSet<String> = command
        .get_subcommands()
        .map(|subcommand| subcommand.get_name().to_owned())
        .collect();
    let listed: BTreeSet<String> = PARSER_SUBCOMMANDS
        .iter()
        .map(|subcommand| (*subcommand).to_owned())
        .collect();

    assert_eq!(listed, actual);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn cli_parser_never_panics_on_arbitrary_args(
        args in prop::collection::vec("\\PC{0,30}", 0..20)
    ) {
        let mut iter_args = vec!["tokmd".to_string()];
        iter_args.extend(args);
        let _ = Cli::try_parse_from(iter_args);
    }

    #[test]
    fn cli_parser_never_panics_on_subcommand_with_arbitrary_args(
        subcmd in parser_subcommands(),
        args in prop::collection::vec("\\PC{0,30}", 0..20)
    ) {
        let mut iter_args = vec!["tokmd".to_string(), subcmd.to_string()];
        iter_args.extend(args);
        let _ = Cli::try_parse_from(iter_args);
    }

    #[test]
    fn cli_parser_never_panics_on_enum_values(
        subcmd in parser_subcommands(),
        key in config_keys(),
        val in "\\PC{0,20}",
        args in prop::collection::vec("\\PC{0,30}", 0..10)
    ) {
        let mut iter_args = vec![
            "tokmd".to_string(),
            subcmd.to_string(),
            key.to_string(),
            val
        ];
        iter_args.extend(args);
        let _ = Cli::try_parse_from(iter_args);
    }
}

#[test]
fn config_resolve_fuzz() {
    use tokmd::ResolvedConfig;
    use tokmd::cli::CliLangArgs;
    use tokmd::resolve_lang_with_config;
    use tokmd_settings::{Profile, ViewProfile};

    proptest!(|(
        cli_top in prop::option::of(0..1000u64),
        view_top in prop::option::of(0..1000u64),
        prof_top in prop::option::of(0..1000u64),
        cli_files in any::<bool>(),
        view_files in prop::option::of(any::<bool>()),
        prof_files in prop::option::of(any::<bool>()),
    )| {
        let cli = CliLangArgs {
            format: None,
            top: cli_top.map(|t| t as usize),
            files: cli_files,
            paths: None,
            children: None,
        };

        let view = ViewProfile {
            top: view_top.map(|t| t as usize),
            format: None,
            files: view_files,
            children: None,
            ..Default::default()
        };

        let profile = Profile {
            top: prof_top.map(|t| t as usize),
            format: None,
            files: prof_files,
            children: None,
            ..Default::default()
        };

        let config = ResolvedConfig {
            toml_view: Some(&view),
            json_profile: Some(&profile),
            toml: None,
        };

        let resolved = resolve_lang_with_config(&cli, &config);

        let expected_top = cli_top.map(|t| t as usize)
            .or(view_top.map(|t| t as usize))
            .or(prof_top.map(|t| t as usize))
            .unwrap_or(0);

        assert_eq!(resolved.top, expected_top);

        // Logical OR priority: CLI, then View, then Profile
        // If CLI is true, it's true.
        // If CLI is false, we check View.
        // If View is Some(true), it's true.
        // If View is Some(false), it's false (it shadows Profile).
        // If View is None, we check Profile.
        // If Profile is Some(true), it's true.
        // If Profile is Some(false) or None, it's false.
        let expected_files = if cli_files {
            true
        } else if let Some(v) = view_files {
            v
        } else if let Some(p) = prof_files {
            p
        } else {
            false
        };
        assert_eq!(resolved.files, expected_files);
    });
}

#[test]
fn config_resolve_fuzz_export() {
    use tokmd::ResolvedConfig;
    use tokmd::cli::CliExportArgs;
    use tokmd::resolve_export_with_config;
    use tokmd_settings::{ExportConfig, TomlConfig};

    proptest!(|(
        cli_min in prop::option::of(0..1000u64),
        toml_min in prop::option::of(0..1000u64),
        cli_max in prop::option::of(0..1000u64),
        toml_max in prop::option::of(0..1000u64),
    )| {
        let cli = CliExportArgs {
            format: None,
            min_code: cli_min.map(|t| t as usize),
            max_rows: cli_max.map(|t| t as usize),
            paths: None,
            output: None,
            module_roots: None,
            module_depth: None,
            children: None,
            redact: None,
            meta: None,
            strip_prefix: None,
        };

        let toml = TomlConfig {
            export: ExportConfig {
                min_code: toml_min.map(|t| t as usize),
                max_rows: toml_max.map(|t| t as usize),
                ..Default::default()
            },
            ..Default::default()
        };

        let config = ResolvedConfig {
            toml_view: None,
            json_profile: None,
            toml: Some(&toml),
        };

        let resolved = resolve_export_with_config(&cli, &config);

        let expected_min = cli_min.map(|t| t as usize)
            .or(toml_min.map(|t| t as usize))
            .unwrap_or(0);

        assert_eq!(resolved.min_code, expected_min);

        let expected_max = cli_max.map(|t| t as usize)
            .or(toml_max.map(|t| t as usize))
            .unwrap_or(0);

        assert_eq!(resolved.max_rows, expected_max);
    });
}

#[test]
fn config_resolve_fuzz_module() {
    use tokmd::ResolvedConfig;
    use tokmd::cli::CliModuleArgs;
    use tokmd::resolve_module_with_config;
    use tokmd_settings::{ModuleConfig, TomlConfig};

    proptest!(|(
        cli_depth in prop::option::of(0..20u8),
        toml_depth in prop::option::of(0..20u8),
        cli_top in prop::option::of(0..1000u64),

    )| {
        let cli = CliModuleArgs {
            format: None,
            top: cli_top.map(|t| t as usize),
            module_depth: cli_depth.map(|d| d as usize),
            paths: None,
            module_roots: None,
            children: None,
        };

        let toml = TomlConfig {
            module: ModuleConfig {
                depth: toml_depth.map(|d| d as usize),
                ..Default::default()
            },
            ..Default::default()
        };

        // Note: the original `resolve_module_with_config` currently seems to not pick up
        // `top` from the view/profile directly via the `TomlConfig` without a `ViewProfile`.
        // We will just test the depth here which is in the `ModuleConfig`.
        let config = ResolvedConfig {
            toml_view: None,
            json_profile: None,
            toml: Some(&toml),
        };

        let resolved = resolve_module_with_config(&cli, &config);

        let expected_depth = cli_depth.map(|d| d as usize)
            .or(toml_depth.map(|d| d as usize))
            .unwrap_or(2); // 2 is the default fallback

        assert_eq!(resolved.module_depth, expected_depth);
    });
}
