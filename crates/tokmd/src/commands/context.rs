use std::path::{Path, PathBuf};

use crate::cli;
use anyhow::Result;
use tokmd_model as model;
use tokmd_scan as scan;
use tokmd_scan::{add_exclude_pattern, normalize_exclude_pattern};
use tokmd_types::ContextExcludedPath;

use crate::context_pack;
use crate::progress::Progress;

pub(crate) fn handle(args: cli::CliContextArgs, global: &cli::GlobalArgs) -> Result<()> {
    let progress = Progress::new(!global.no_progress);

    let paths = args
        .paths
        .clone()
        .unwrap_or_else(|| vec![PathBuf::from(".")]);

    // Parse budget
    let budget = context_pack::parse_budget(&args.budget)?;

    let root = paths.first().cloned().unwrap_or_else(|| PathBuf::from("."));

    // Scan and create export data
    progress.set_message("Scanning codebase...");
    let mut scan_args = global.clone();
    let mut excluded_paths: Vec<ContextExcludedPath> = Vec::new();
    add_excluded_path(
        &root,
        args.output.as_ref(),
        "out_file",
        &mut scan_args,
        &mut excluded_paths,
    );
    add_excluded_path(
        &root,
        args.bundle_dir.as_ref(),
        "bundle_dir",
        &mut scan_args,
        &mut excluded_paths,
    );
    add_excluded_path(
        &root,
        args.log.as_ref(),
        "log_file",
        &mut scan_args,
        &mut excluded_paths,
    );
    let scan_opts = tokmd_settings::ScanOptions::from(&scan_args);
    let languages = scan::scan(&paths, &scan_opts)?;
    let module_roots = args.module_roots.clone().unwrap_or_default();
    let module_depth = args.module_depth.unwrap_or(2);

    progress.set_message("Building export data...");
    let export = model::create_export_data(
        &languages,
        &module_roots,
        module_depth,
        tokmd_types::ChildIncludeMode::ParentsOnly,
        None,
        0, // no min_code filter
        0, // no max_rows limit
    );

    // Compute git scores if using churn/hotspot ranking
    progress.set_message("Computing scores...");
    let needs_git = matches!(
        args.rank_by,
        cli::ValueMetric::Churn | cli::ValueMetric::Hotspot
    );
    let git_scores = if needs_git && !args.no_git {
        let root = paths.first().cloned().unwrap_or_else(|| PathBuf::from("."));
        match tokmd_core::context_git::compute_git_scores(
            &root,
            &export.rows,
            args.max_commits,
            args.max_commit_files,
        ) {
            Some(scores) => {
                if scores.hotspots.is_empty() && args.git {
                    eprintln!("Warning: no git history found for scanned files");
                }
                Some(scores)
            }
            None => {
                if args.git {
                    eprintln!("Warning: git data unavailable, falling back to code lines");
                }
                None
            }
        }
    } else {
        None
    };

    // Select files based on strategy
    progress.set_message("Selecting files for context...");
    let select_result = context_pack::select_files_with_options(
        &export.rows,
        budget,
        args.strategy,
        args.rank_by,
        git_scores.as_ref(),
        &context_pack::SelectOptions {
            no_smart_exclude: args.no_smart_exclude,
            max_file_pct: args.max_file_pct,
            max_file_tokens: args.max_file_tokens,
            require_git_scores: args.require_git_scores,
            ..Default::default()
        },
    );

    // Error if require_git_scores is set and a fallback occurred
    if args.require_git_scores && select_result.fallback_reason.is_some() {
        anyhow::bail!(
            "Git scores required but unavailable: {}",
            select_result
                .fallback_reason
                .as_deref()
                .unwrap_or("unknown")
        );
    }

    let selected = &select_result.selected;

    let used_tokens: usize = selected
        .iter()
        .map(|f| f.effective_tokens.unwrap_or(f.tokens))
        .sum();
    let utilization = if budget > 0 {
        (used_tokens as f64 / budget as f64) * 100.0
    } else {
        0.0
    };

    progress.finish_and_clear();

    // Determine output destination for logging
    let output_destination = context_pack::determine_output_destination(&args);

    // Write output and get total bytes written
    let total_bytes = if let Some(ref bundle_dir) = args.bundle_dir {
        // Handle bundle directory mode - streams directly to files
        context_pack::write_bundle_directory(
            bundle_dir,
            &args,
            selected,
            budget,
            used_tokens,
            utilization,
            args.force,
            &excluded_paths,
            &scan_args.excluded,
            &select_result,
        )?
    } else {
        // For bundle output mode, stream directly to destination
        // For list/json output modes, build string (small outputs)
        context_pack::write_to_destination(
            &args,
            selected,
            budget,
            used_tokens,
            utilization,
            &select_result,
        )?
    };

    // Check size threshold and emit warning if exceeded (after writing)
    let max_bytes = args.max_output_bytes;
    if max_bytes > 0 && total_bytes as u64 > max_bytes {
        eprintln!(
            "Warning: output size ({} bytes) exceeds threshold ({} bytes). Consider using --bundle-dir for large outputs.",
            total_bytes, max_bytes
        );
    }

    // Handle log append
    if let Some(ref log_path) = args.log {
        context_pack::append_context_log_record(
            log_path,
            &args,
            budget,
            used_tokens,
            utilization,
            selected.len(),
            total_bytes,
            output_destination,
        )?;
    }

    Ok(())
}

fn add_excluded_path(
    root: &Path,
    path: Option<&PathBuf>,
    reason: &str,
    scan_args: &mut cli::GlobalArgs,
    excluded_paths: &mut Vec<ContextExcludedPath>,
) {
    let Some(path) = path else { return };
    let pattern = normalize_exclude_pattern(root, path);
    if pattern.is_empty() {
        return;
    }

    let _ = add_exclude_pattern(&mut scan_args.excluded, pattern.clone());

    if !excluded_paths.iter().any(|p| p.path == pattern) {
        excluded_paths.push(ContextExcludedPath {
            path: pattern,
            reason: reason.to_string(),
        });
    }
}
