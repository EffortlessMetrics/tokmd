use tokmd_config as cli;
use tokmd_format as format;
use tokmd_model as model;
use tokmd_scan as scan;
use tokmd_tokeignore as tokeignore;

use anyhow::Result;
use clap::{CommandFactory, Parser};

use cli::{Cli, Commands};

/// Entry point used by the `tokmd` (and optional `tok`) binaries.
pub fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command.unwrap_or(Commands::Lang(cli.lang.clone())) {
        Commands::Completions(args) => {
            use clap_complete::generate;
            let mut cmd = Cli::command();
            let name = cmd.get_name().to_string();
            let shell = match args.shell {
                cli::Shell::Bash => clap_complete::Shell::Bash,
                cli::Shell::Elvish => clap_complete::Shell::Elvish,
                cli::Shell::Fish => clap_complete::Shell::Fish,
                cli::Shell::Powershell => clap_complete::Shell::PowerShell,
                cli::Shell::Zsh => clap_complete::Shell::Zsh,
            };
            generate(shell, &mut cmd, name, &mut std::io::stdout());
        }
        Commands::Lang(args) => {
            let languages = scan::scan(&args.paths, &cli.global)?;
            let report =
                model::create_lang_report(&languages, args.top, args.files, args.children);
            format::print_lang_report(&report, &cli.global, &args)?;
        }
        Commands::Module(args) => {
            let languages = scan::scan(&args.paths, &cli.global)?;
            let report = model::create_module_report(
                &languages,
                &args.module_roots,
                args.module_depth,
                args.children,
                args.top,
            );
            format::print_module_report(&report, &cli.global, &args)?;
        }
        Commands::Export(args) => {
            let languages = scan::scan(&args.paths, &cli.global)?;
            let strip_prefix = args.strip_prefix.as_deref();
            let export = model::create_export_data(
                &languages,
                &args.module_roots,
                args.module_depth,
                args.children,
                strip_prefix,
                args.min_code,
                args.max_rows,
            );
            format::write_export(&export, &cli.global, &args)?;
        }
        Commands::Init(args) => {
            tokeignore::init_tokeignore(&args)?;
        }
    }

    Ok(())
}
