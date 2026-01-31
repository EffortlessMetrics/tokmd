use anyhow::Result;
use clap::Parser;

mod cli;
mod tasks;

use cli::{PublishArgs, XtaskCli};

fn main() -> Result<()> {
    let cli = XtaskCli::parse();

    match cli.command {
        Some(cli::Commands::Publish(args)) => tasks::publish::run(args),
        Some(cli::Commands::Cockpit(args)) => tasks::cockpit::run(args),
        None => tasks::publish::run(PublishArgs::default()),
    }
}
