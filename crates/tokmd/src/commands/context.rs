use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::cli;
use anyhow::{Context, Result};
use tokmd_model as model;
use tokmd_scan as scan;
use tokmd_scan::{add_exclude_pattern, normalize_exclude_pattern};
use tokmd_types::{
    CONTEXT_SCHEMA_VERSION, ContextExcludedPath, ContextFileRow, ContextLogRecord, ContextReceipt,
    SCHEMA_VERSION, ToolInfo,
};

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
    let output_destination = determine_output_destination(&args);

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
        write_to_destination(
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
        let log_record = ContextLogRecord {
            schema_version: SCHEMA_VERSION,
            generated_at_ms: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis(),
            tool: ToolInfo::current(),
            budget_tokens: budget,
            used_tokens,
            utilization_pct: utilization,
            strategy: format!("{:?}", args.strategy).to_lowercase(),
            rank_by: format!("{:?}", args.rank_by).to_lowercase(),
            file_count: selected.len(),
            total_bytes,
            output_destination,
        };
        append_log_record(log_path, &log_record)?;
    }

    Ok(())
}

/// Determine the output destination string for logging.
fn determine_output_destination(args: &cli::CliContextArgs) -> String {
    if let Some(ref bundle_dir) = args.bundle_dir {
        format!("bundle:{}", bundle_dir.display())
    } else if let Some(ref out_path) = args.output {
        format!("file:{}", out_path.display())
    } else {
        "stdout".to_string()
    }
}

/// Write output to destination and return total bytes written.
/// For bundle output, streams directly to avoid memory blowup.
/// For list/json output, builds string first (small outputs).
fn write_to_destination(
    args: &cli::CliContextArgs,
    selected: &[ContextFileRow],
    budget: usize,
    used_tokens: usize,
    utilization: f64,
    select_result: &context_pack::SelectResult,
) -> Result<usize> {
    match args.output_mode {
        cli::ContextOutput::Bundle => {
            // Stream bundle output directly to destination
            write_bundle_to_destination(args, selected)
        }
        cli::ContextOutput::List | cli::ContextOutput::Json => {
            // Build string for list/json (small outputs)
            let content = match args.output_mode {
                cli::ContextOutput::List => context_pack::format_list_output(
                    selected,
                    budget,
                    used_tokens,
                    utilization,
                    args.strategy,
                ),
                cli::ContextOutput::Json => format_json_output(
                    selected,
                    budget,
                    used_tokens,
                    utilization,
                    args,
                    select_result,
                )?,
                cli::ContextOutput::Bundle => unreachable!(),
            };
            let total_bytes = content.len();

            if let Some(ref out_path) = args.output {
                write_output_file(out_path, &content, args.force)?;
            } else {
                print!("{}", content);
            }

            Ok(total_bytes)
        }
    }
}

/// Write bundle output directly to destination (file or stdout).
/// Streams content to avoid loading entire bundle into memory.
fn write_bundle_to_destination(
    args: &cli::CliContextArgs,
    selected: &[ContextFileRow],
) -> Result<usize> {
    if let Some(ref out_path) = args.output {
        // Open file with proper semantics: create_new fails if exists (unless --force)
        let file = if args.force {
            OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(out_path)
        } else {
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(out_path)
        }
        .with_context(|| {
            if !args.force && out_path.exists() {
                format!(
                    "Output file already exists: {}. Use --force to overwrite.",
                    out_path.display()
                )
            } else {
                format!("Failed to create output file: {}", out_path.display())
            }
        })?;

        let mut counter = context_pack::CountingWriter::new(file);
        context_pack::write_bundle_output(&mut counter, selected, args.compress)?;
        counter.flush()?;

        let bytes = counter.bytes() as usize;
        eprintln!("Wrote {}", out_path.display());
        Ok(bytes)
    } else {
        // Stream to stdout
        let stdout = std::io::stdout();
        let mut counter = context_pack::CountingWriter::new(stdout.lock());
        context_pack::write_bundle_output(&mut counter, selected, args.compress)?;
        counter.flush()?;
        Ok(counter.bytes() as usize)
    }
}

/// Format JSON receipt output.
fn format_json_output(
    selected: &[ContextFileRow],
    budget: usize,
    used_tokens: usize,
    utilization: f64,
    args: &cli::CliContextArgs,
    select_result: &context_pack::SelectResult,
) -> Result<String> {
    let total_file_bytes: usize = selected.iter().map(|f| f.bytes).sum();
    let token_estimation = tokmd_types::TokenEstimationMeta::from_bytes(total_file_bytes, 4.0);
    let receipt = ContextReceipt {
        schema_version: CONTEXT_SCHEMA_VERSION,
        generated_at_ms: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis(),
        tool: ToolInfo::current(),
        mode: "context".to_string(),
        budget_tokens: budget,
        used_tokens,
        utilization_pct: utilization,
        strategy: format!("{:?}", args.strategy).to_lowercase(),
        rank_by: format!("{:?}", args.rank_by).to_lowercase(),
        file_count: selected.len(),
        files: selected.to_vec(),
        rank_by_effective: if select_result.fallback_reason.is_some() {
            Some(select_result.rank_by_effective.clone())
        } else {
            None
        },
        fallback_reason: select_result.fallback_reason.clone(),
        excluded_by_policy: select_result.excluded_by_policy.clone(),
        token_estimation: Some(token_estimation),
        bundle_audit: None,
    };
    let json = serde_json::to_string_pretty(&receipt)?;
    Ok(format!("{}\n", json))
}

/// Write output to a file, checking for existence unless force is true.
fn write_output_file(path: &Path, content: &str, force: bool) -> Result<()> {
    // Open file with proper semantics: create_new fails if exists (unless --force)
    let mut file = if force {
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
    } else {
        OpenOptions::new().write(true).create_new(true).open(path)
    }
    .with_context(|| {
        if !force && path.exists() {
            format!(
                "Output file already exists: {}. Use --force to overwrite.",
                path.display()
            )
        } else {
            format!("Failed to write output file: {}", path.display())
        }
    })?;

    file.write_all(content.as_bytes())
        .with_context(|| format!("Failed to write output file: {}", path.display()))?;
    eprintln!("Wrote {}", path.display());
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

/// Append a log record to a JSONL file.
fn append_log_record(path: &Path, record: &ContextLogRecord) -> Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("Failed to open log file: {}", path.display()))?;

    let json = serde_json::to_string(record)?;
    writeln!(file, "{}", json)
        .with_context(|| format!("Failed to append to log file: {}", path.display()))?;

    Ok(())
}
