use anyhow::Result;
use tokmd_config as cli;
use tokmd_format as format;
use tokmd_model as model;
use tokmd_scan as scan;

use crate::config;

pub(crate) fn handle(
    cli_args: cli::CliModuleArgs,
    global: &cli::GlobalArgs,
    profile: Option<&cli::Profile>,
) -> Result<()> {
    let args = config::resolve_module(&cli_args, profile);
    let languages = scan::scan(&args.paths, global)?;
    let report = model::create_module_report(
        &languages,
        &args.module_roots,
        args.module_depth,
        args.children,
        args.top,
    );
    format::print_module_report(&report, global, &args)?;
    Ok(())
}
