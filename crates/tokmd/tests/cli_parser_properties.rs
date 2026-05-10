//! Property-based tests for CLI parser invariants.
//!
//! Verifies that the clap parser (`tokmd::cli::Cli`) never panics
//! when fed arbitrary string arguments.

use proptest::prelude::*;
use tokmd::cli::Cli;
use clap::Parser;

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
        subcmd in prop_oneof![
            Just("lang"),
            Just("module"),
            Just("export"),
            Just("diff"),
            Just("version"),
            Just("analyze"),
            Just("cockpit"),
            Just("context"),
            Just("handoff"),
        ],
        args in prop::collection::vec("\\PC{0,30}", 0..20)
    ) {
        let mut iter_args = vec!["tokmd".to_string(), subcmd.to_string()];
        iter_args.extend(args);
        let _ = Cli::try_parse_from(iter_args);
    }
}
