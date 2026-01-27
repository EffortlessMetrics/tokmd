use anyhow::Result;
use tokmd_config as cli;
use tokmd_format as format;
use tokmd_model as model;
use tokmd_scan as scan;

use crate::config;

pub(crate) fn handle(
    cli_args: cli::CliExportArgs,
    global: &cli::GlobalArgs,
    profile: Option<&cli::Profile>,
) -> Result<()> {
    let args = config::resolve_export(&cli_args, profile);
    let languages = scan::scan(&args.paths, global)?;
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
    format::write_export(&export, global, &args)?;
    Ok(())
}
