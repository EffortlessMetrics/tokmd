use anyhow::Result;
use tokmd_config as cli;
use tokmd_format as format;
use tokmd_model as model;
use tokmd_scan as scan;
use tokmd_settings::ScanOptions;

use crate::config::{self, ResolvedConfig};

pub(crate) fn handle(
    cli_args: cli::CliLangArgs,
    global: &cli::GlobalArgs,
    resolved: &ResolvedConfig,
) -> Result<()> {
    let args = config::resolve_lang_with_config(&cli_args, resolved);
    let scan_opts = ScanOptions::from(global);
    let languages = scan::scan(&args.paths, &scan_opts)?;
    let metrics = model::compute_file_metrics(&languages);
    let report =
        model::create_lang_report(&languages, &metrics, args.top, args.files, args.children);
    format::print_lang_report(&report, &scan_opts, &args)?;
    Ok(())
}
