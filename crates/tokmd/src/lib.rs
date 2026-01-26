use tokmd_config as cli;
use tokmd_format as format;
use tokmd_model as model;
use tokmd_scan as scan;
use tokmd_tokeignore as tokeignore;

use anyhow::Result;
use clap::Parser;

use cli::{Cli, Commands};

/// Entry point used by the `tokmd` (and optional `tok`) binaries.
pub fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command.unwrap_or(Commands::Lang(cli.lang.clone())) {
        Commands::Lang(args) => {
            let languages = scan::scan(&args.paths, &cli.global)?;
            let report =
                model::LangReport::from_languages(&languages, args.top, args.files, args.children);
            format::print_lang_report(&report, &cli.global, &args)?;
        }
        Commands::Module(args) => {
            let languages = scan::scan(&args.paths, &cli.global)?;
            let report = model::ModuleReport::from_languages(
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
            let export = model::ExportData::from_languages(
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
