//! Top-level clap subcommand enum.
//!
//! This module owns command names, aliases, and subcommand argument wiring. The
//! execution dispatcher remains in `crate::commands`.

use clap::Subcommand;

use super::{
    BadgeArgs, BaselineArgs, CliAnalyzeArgs, CliCheckIgnoreArgs, CliContextArgs, CliExportArgs,
    CliGateArgs, CliLangArgs, CliModuleArgs, CockpitArgs, CompletionsArgs, DiffArgs, HandoffArgs,
    InitArgs, RunArgs, SensorArgs, ToolsArgs,
};

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Language summary (default).
    Lang(CliLangArgs),

    /// Module summary (group by path prefixes like `crates/<name>` or `packages/<name>`).
    Module(CliModuleArgs),

    /// Export a file-level dataset (CSV / JSONL / JSON).
    Export(CliExportArgs),

    /// Analyze receipts or paths to produce derived metrics.
    #[command(visible_alias = "analyse")]
    Analyze(CliAnalyzeArgs),

    /// Render a simple SVG badge for a metric.
    Badge(BadgeArgs),

    /// Write a `.tokeignore` template to the target directory.
    Init(InitArgs),

    /// Generate shell completions.
    #[command(visible_alias = "completion")]
    Completions(CompletionsArgs),

    /// Run a full scan and save receipts to a state directory.
    Run(RunArgs),

    /// Compare two receipts or runs.
    Diff(DiffArgs),

    /// Pack files into an LLM context window within a token budget.
    Context(CliContextArgs),

    /// Check why a file is being ignored (for troubleshooting).
    CheckIgnore(CliCheckIgnoreArgs),

    /// Output CLI schema as JSON for AI agents.
    Tools(ToolsArgs),

    /// Evaluate policy rules against analysis receipts.
    Gate(CliGateArgs),

    /// Generate PR cockpit metrics for code review.
    Cockpit(CockpitArgs),

    /// Generate a complexity baseline for trend tracking.
    Baseline(BaselineArgs),

    /// Bundle codebase for LLM handoff.
    Handoff(HandoffArgs),

    /// Run as a conforming sensor, producing a SensorReport.
    Sensor(SensorArgs),
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::*;
    use crate::cli::parser::Cli;

    #[test]
    fn depth_visible_alias_sets_module_depth_for_module_like_commands() {
        let cli = Cli::try_parse_from(["tokmd", "module", "--depth", "3"]).unwrap();
        match cli.command.unwrap() {
            Commands::Module(args) => assert_eq!(args.module_depth, Some(3)),
            other => panic!("unexpected command: {other:?}"),
        }

        let cli = Cli::try_parse_from(["tokmd", "export", "--depth", "3"]).unwrap();
        match cli.command.unwrap() {
            Commands::Export(args) => assert_eq!(args.module_depth, Some(3)),
            other => panic!("unexpected command: {other:?}"),
        }

        let cli = Cli::try_parse_from(["tokmd", "context", "--depth", "3"]).unwrap();
        match cli.command.unwrap() {
            Commands::Context(args) => assert_eq!(args.module_depth, Some(3)),
            other => panic!("unexpected command: {other:?}"),
        }

        let cli = Cli::try_parse_from(["tokmd", "handoff", "--depth", "3"]).unwrap();
        match cli.command.unwrap() {
            Commands::Handoff(args) => assert_eq!(args.module_depth, Some(3)),
            other => panic!("unexpected command: {other:?}"),
        }
    }
}
