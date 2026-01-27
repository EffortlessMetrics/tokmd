use anyhow::Result;
use tokmd_config as cli;
use tokmd_format as format;
use tokmd_model as model;
use tokmd_scan as scan;

use crate::config;

pub(crate) fn handle(
    cli_args: cli::CliLangArgs,
    global: &cli::GlobalArgs,
    profile: Option<&cli::Profile>,
) -> Result<()> {
    let args = config::resolve_lang(&cli_args, profile);
    let languages = scan::scan(&args.paths, global)?;
    let report = model::create_lang_report(&languages, args.top, args.files, args.children);
    format::print_lang_report(&report, global, &args)?;
    Ok(())
}
