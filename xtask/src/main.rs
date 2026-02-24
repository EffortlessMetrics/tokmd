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
        Some(cli::Commands::BoundariesCheck(args)) => tasks::boundaries_check::run(args),
        None => tasks::publish::run(PublishArgs::default()),
    }
}
