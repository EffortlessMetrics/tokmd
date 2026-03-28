use anyhow::Result;
use clap::Parser;

mod cli;
mod tasks;

use cli::{PublishArgs, XtaskCli};

fn main() -> Result<()> {
    let cli = XtaskCli::parse();

    match cli.command {
        Some(cli::Commands::Bump(args)) => tasks::bump::run(args),
        Some(cli::Commands::Publish(args)) => tasks::publish::run(args),
        Some(cli::Commands::Cockpit(args)) => tasks::cockpit::run(args),
        Some(cli::Commands::Docs(args)) => tasks::docs::run(args),
        Some(cli::Commands::VersionConsistency(args)) => tasks::version_consistency::run(args),
        Some(cli::Commands::BoundariesCheck(args)) => tasks::boundaries_check::run(args),
        Some(cli::Commands::FixtureBlobsCheck(args)) => tasks::fixture_blobs_check::run(args),
        Some(cli::Commands::Gate(args)) => tasks::gate::run(args),
        Some(cli::Commands::LintFix(args)) => tasks::lint_fix::run(args),
        Some(cli::Commands::Sccache(args)) => tasks::sccache::run(args),
        Some(cli::Commands::TrimTarget(args)) => tasks::trim_target::run(args),
        None => tasks::publish::run(PublishArgs::default()),
    }
}
