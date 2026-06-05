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
    "evidence-packet",
];

fn parser_subcommands() -> impl Strategy<Value = &'static str> {
    prop::sample::select(PARSER_SUBCOMMANDS.to_vec())
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
}
