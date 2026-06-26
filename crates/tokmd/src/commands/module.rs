use crate::cli;
use anyhow::Result;
use tokmd_format as format;
use tokmd_model as model;
use tokmd_scan as scan;
use tokmd_settings::ScanOptions;

use crate::config::{self, ResolvedConfig};
use crate::progress::Progress;

pub(crate) fn handle(
    cli_args: cli::CliModuleArgs,
    global: &cli::GlobalArgs,
    resolved: &ResolvedConfig,
) -> Result<()> {
    let args = config::resolve_module_with_config(&cli_args, resolved);
    let scan_opts = ScanOptions::from(global);

    let progress = Progress::new(!global.no_progress);
    progress.set_message("Scanning codebase...");
    let languages = scan::scan(&args.paths, &scan_opts)?;
    let report = model::create_module_report(
        &languages,
        &args.module_roots,
        args.module_depth,
        args.children,
        args.top,
    );
    // Clear the stderr spinner before the report is written to stdout.
    progress.finish_and_clear();

    format::print_module_report(&report, &scan_opts, &args)?;
    Ok(())
}
