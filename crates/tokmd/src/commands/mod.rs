pub(crate) mod analyze;
pub(crate) mod badge;
pub(crate) mod check_ignore;
pub(crate) mod completions;
pub(crate) mod context;
pub(crate) mod diff;
pub(crate) mod export;
pub(crate) mod init;
pub(crate) mod lang;
pub(crate) mod module;
pub(crate) mod run;

use anyhow::Result;
use tokmd_config as cli;

use crate::config::ResolvedConfig;

pub(crate) fn dispatch(cli: cli::Cli, resolved: &ResolvedConfig) -> Result<()> {
    let global = &cli.global;
    match cli.command.unwrap_or(cli::Commands::Lang(cli.lang.clone())) {
        cli::Commands::Completions(args) => completions::handle(args),
        cli::Commands::Run(args) => run::handle(args, global),
        cli::Commands::Diff(args) => diff::handle(args, global),
        cli::Commands::Lang(args) => lang::handle(args, global, resolved),
        cli::Commands::Module(args) => module::handle(args, global, resolved),
        cli::Commands::Export(args) => export::handle(args, global, resolved),
        cli::Commands::Analyze(args) => analyze::handle(args, global),
        cli::Commands::Badge(args) => badge::handle(args, global),
        cli::Commands::Init(args) => init::handle(args),
        cli::Commands::Context(args) => context::handle(args, global),
        cli::Commands::CheckIgnore(args) => check_ignore::handle(args, global),
    }
}
