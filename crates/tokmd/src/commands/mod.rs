#[cfg(feature = "analysis")]
pub(crate) mod analyze;
#[cfg(feature = "analysis")]
pub(crate) mod badge;
#[cfg(feature = "analysis")]
pub(crate) mod baseline;
pub(crate) mod check_ignore;
pub(crate) mod cockpit;
pub(crate) mod completions;
pub(crate) mod context;
pub(crate) mod diff;
pub(crate) mod evidence_packet;
pub(crate) mod export;
#[cfg(feature = "analysis")]
pub(crate) mod gate;
pub(crate) mod handoff;
pub(crate) mod init;
pub(crate) mod lang;
pub(crate) mod module;
#[cfg(feature = "analysis")]
pub(crate) mod packet;
#[cfg(feature = "analysis")]
pub(crate) mod render;
pub(crate) mod run;
pub(crate) mod sensor;
#[cfg(feature = "ast")]
pub(crate) mod syntax;
pub(crate) mod tools;

use crate::cli;
use anyhow::{Error, Result};

use crate::config::ResolvedConfig;

#[derive(Debug)]
pub(crate) struct UsageError(String);

impl std::fmt::Display for UsageError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for UsageError {}

fn reject_implicit_lang_options_with_subcommand(cli: &cli::Cli) -> Result<()> {
    if cli.command.is_some()
        && (cli.lang.format.is_some()
            || cli.lang.top.is_some()
            || cli.lang.files
            || cli.lang.children.is_some())
    {
        return Err(Error::new(UsageError(
            "default `lang` options cannot appear before a subcommand; move them after the command (for example, `tokmd module --format json`)".to_string(),
        )));
    }
    Ok(())
}

pub(crate) fn dispatch(cli: cli::Cli, resolved: &ResolvedConfig) -> Result<()> {
    reject_implicit_lang_options_with_subcommand(&cli)?;
    let global = &cli.global;
    match cli.command.unwrap_or(cli::Commands::Lang(cli.lang.clone())) {
        cli::Commands::Completions(args) => completions::handle(args),
        #[cfg(feature = "analysis")]
        cli::Commands::Run(args) => run::handle(args, global),
        cli::Commands::Diff(args) => diff::handle(args, global),
        cli::Commands::Lang(args) => lang::handle(args, global, resolved),
        cli::Commands::Module(args) => module::handle(args, global, resolved),
        cli::Commands::Export(args) => export::handle(args, global, resolved),
        #[cfg(feature = "analysis")]
        cli::Commands::Analyze(args) => analyze::handle(args, global),
        #[cfg(feature = "analysis")]
        cli::Commands::Badge(args) => badge::handle(args, global),
        cli::Commands::Init(args) => init::handle(args),
        cli::Commands::Context(args) => context::handle(args, global),
        cli::Commands::CheckIgnore(args) => check_ignore::handle(args, global),
        cli::Commands::Tools(args) => tools::handle(args),
        #[cfg(feature = "analysis")]
        cli::Commands::Gate(args) => gate::handle(args, global, resolved),
        cli::Commands::Cockpit(args) => cockpit::handle(args, global),
        #[cfg(feature = "analysis")]
        cli::Commands::Baseline(args) => baseline::handle(args, global),
        cli::Commands::Handoff(args) => handoff::handle(args, global),
        cli::Commands::Sensor(args) => sensor::handle(args, global),
        #[cfg(feature = "ast")]
        cli::Commands::Syntax(args) => syntax::handle(args, global),
        cli::Commands::EvidencePacket(args) => evidence_packet::handle(args),
        cli::Commands::Render(args) => render::handle(args),
        #[cfg(feature = "analysis")]
        cli::Commands::Packet(args) => packet::handle(args, global),
        #[cfg(not(feature = "analysis"))]
        _ => anyhow::bail!("analysis feature is not enabled"),
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::*;

    #[test]
    fn rejects_default_lang_options_before_a_subcommand() -> Result<()> {
        let cases: &[&[&str]] = &[
            &["tokmd", "--format", "json", "module"],
            &["tokmd", "--top", "5", "module"],
            &["tokmd", "--files", "module"],
            &["tokmd", "--children", "separate", "module"],
            &["tokmd", "--format", "json", "module", "--format", "md"],
            &["tokmd", "--files", "analyze"],
        ];

        for args in cases {
            let parsed = cli::Cli::try_parse_from(*args)?;
            if reject_implicit_lang_options_with_subcommand(&parsed).is_ok() {
                return Err(anyhow::anyhow!(
                    "accepted ignored root option case: {args:?}"
                ));
            }
        }
        Ok(())
    }

    #[test]
    fn accepts_lang_options_after_a_subcommand() -> Result<()> {
        let cases: &[&[&str]] = &[
            &["tokmd", "module", "--format", "json"],
            &["tokmd", "module", "--top", "5"],
            &["tokmd", "module", "--children", "separate"],
        ];
        for args in cases {
            let parsed = cli::Cli::try_parse_from(*args)?;
            reject_implicit_lang_options_with_subcommand(&parsed)?;
        }
        Ok(())
    }

    #[test]
    fn module_subcommand_rejects_lang_only_files_flag() -> Result<()> {
        if cli::Cli::try_parse_from(["tokmd", "module", "--files"]).is_ok() {
            return Err(anyhow::anyhow!(
                "module unexpectedly accepted unsupported --files"
            ));
        }
        Ok(())
    }
}
