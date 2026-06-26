use crate::cli;
use anyhow::Result;
use tokmd_format as format;
use tokmd_model as model;
use tokmd_scan as scan;
use tokmd_settings::ScanOptions;

use crate::config::{self, ResolvedConfig};
use crate::progress::Progress;

pub(crate) fn handle(
    cli_args: cli::CliExportArgs,
    global: &cli::GlobalArgs,
    resolved: &ResolvedConfig,
) -> Result<()> {
    let args = config::resolve_export_with_config(&cli_args, resolved);
    let scan_opts = ScanOptions::from(global);

    let progress = Progress::new(!global.no_progress);
    progress.set_message("Scanning codebase...");
    let languages = scan::scan(&args.paths, &scan_opts)?;

    progress.set_message("Building file inventory...");
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
    // Clear the stderr spinner before machine-readable output is written so the
    // inventory on stdout stays clean.
    progress.finish_and_clear();

    format::write_export(&export, &scan_opts, &args)?;
    Ok(())
}
