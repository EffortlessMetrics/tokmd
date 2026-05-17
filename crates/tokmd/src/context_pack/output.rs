//! Context command output destination and log writing helpers.

use anyhow::{Context, Result};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use tokmd_types::{ContextFileRow, ContextLogRecord, SCHEMA_VERSION, ToolInfo};

use crate::cli;

use super::{
    CountingWriter, SelectResult, format_list_output,
    receipt::{ContextReceiptParams, build_context_receipt, generated_at_ms, lower_debug},
    write_bundle_output,
};

/// Determine the output destination string for context logging.
pub(crate) fn determine_output_destination(args: &cli::CliContextArgs) -> String {
    if let Some(ref bundle_dir) = args.bundle_dir {
        format!("bundle:{}", bundle_dir.display())
    } else if let Some(ref out_path) = args.output {
        format!("file:{}", out_path.display())
    } else {
        "stdout".to_string()
    }
}

/// Write context output to its configured destination and return bytes written.
///
/// Bundle output streams directly to avoid memory blowup. List and JSON output
/// are built first because they are bounded receipt-like outputs.
pub(crate) fn write_to_destination(
    args: &cli::CliContextArgs,
    selected: &[ContextFileRow],
    budget: usize,
    used_tokens: usize,
    utilization: f64,
    select_result: &SelectResult,
) -> Result<usize> {
    match args.output_mode {
        cli::ContextOutput::Bundle => write_bundle_to_destination(args, selected),
        cli::ContextOutput::List | cli::ContextOutput::Json => {
            let content = match args.output_mode {
                cli::ContextOutput::List => {
                    format_list_output(selected, budget, used_tokens, utilization, args.strategy)
                }
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
                print!("{content}");
            }

            Ok(total_bytes)
        }
    }
}

/// Append a context JSONL log record.
#[allow(clippy::too_many_arguments)]
pub(crate) fn append_context_log_record(
    path: &Path,
    args: &cli::CliContextArgs,
    budget: usize,
    used_tokens: usize,
    utilization: f64,
    file_count: usize,
    total_bytes: usize,
    output_destination: String,
) -> Result<()> {
    let log_record = ContextLogRecord {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: generated_at_ms(),
        tool: ToolInfo::current(),
        budget_tokens: budget,
        used_tokens,
        utilization_pct: utilization,
        strategy: lower_debug(&args.strategy),
        rank_by: lower_debug(&args.rank_by),
        file_count,
        total_bytes,
        output_destination,
    };
    append_log_record(path, &log_record)
}

fn write_bundle_to_destination(
    args: &cli::CliContextArgs,
    selected: &[ContextFileRow],
) -> Result<usize> {
    if let Some(ref out_path) = args.output {
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

        let mut counter = CountingWriter::new(file);
        write_bundle_output(&mut counter, selected, args.compress)?;
        counter.flush()?;

        let bytes = counter.bytes() as usize;
        eprintln!("Wrote {}", out_path.display());
        Ok(bytes)
    } else {
        let stdout = std::io::stdout();
        let mut counter = CountingWriter::new(stdout.lock());
        write_bundle_output(&mut counter, selected, args.compress)?;
        counter.flush()?;
        Ok(counter.bytes() as usize)
    }
}

fn format_json_output(
    selected: &[ContextFileRow],
    budget: usize,
    used_tokens: usize,
    utilization: f64,
    args: &cli::CliContextArgs,
    select_result: &SelectResult,
) -> Result<String> {
    let receipt = build_context_receipt(ContextReceiptParams {
        args,
        selected,
        budget,
        used_tokens,
        utilization,
        select_result,
        generated_at_ms: generated_at_ms(),
        bundle_audit: None,
    });
    let json = serde_json::to_string_pretty(&receipt)?;
    Ok(format!("{json}\n"))
}

fn write_output_file(path: &Path, content: &str, force: bool) -> Result<()> {
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

fn append_log_record(path: &Path, record: &ContextLogRecord) -> Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("Failed to open log file: {}", path.display()))?;

    let json = serde_json::to_string(record)?;
    writeln!(file, "{json}")
        .with_context(|| format!("Failed to append to log file: {}", path.display()))?;

    Ok(())
}
