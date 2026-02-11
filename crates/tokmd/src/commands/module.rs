use anyhow::Result;
use tokmd_config as cli;
use tokmd_format as format;
use tokmd_model as model;
use tokmd_scan as scan;
use tokmd_settings::ScanOptions;

use crate::config::{self, ResolvedConfig};

pub(crate) fn handle(
    cli_args: cli::CliModuleArgs,
    global: &cli::GlobalArgs,
    resolved: &ResolvedConfig,
) -> Result<()> {
    let args = config::resolve_module_with_config(&cli_args, resolved);
    let scan_opts = ScanOptions::from(global);
    let languages = scan::scan(&args.paths, &scan_opts)?;
    let metrics = model::compute_file_metrics(&languages);
    let report = model::create_module_report(
        &languages,
        &metrics,
        &args.module_roots,
        args.module_depth,
        args.children,
        args.top,
    );
    format::print_module_report(&report, &scan_opts, &args)?;
    Ok(())
}
