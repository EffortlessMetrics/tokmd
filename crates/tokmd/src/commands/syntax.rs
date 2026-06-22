//! Feature-gated syntax receipt producer.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde_json::{Value, json};
use tokmd_analysis::ast::{SyntaxParseOptions, SyntaxParseReceipt, parse_syntax_receipt};

use crate::cli;

const SYNTAX_RECEIPTS_SCHEMA: &str = "tokmd.syntax_receipts.v1";

pub(crate) fn handle(args: cli::SyntaxArgs) -> Result<()> {
    let packet = build_syntax_packet(&args)?;
    println!("{}", serde_json::to_string_pretty(&packet)?);

    if let Some(errors) = packet.get("errors").and_then(Value::as_array)
        && !errors.is_empty()
    {
        let detail = errors
            .iter()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>()
            .join("; ");
        bail!("syntax receipts failed: {detail}");
    }

    Ok(())
}

pub(crate) fn build_syntax_packet(args: &cli::SyntaxArgs) -> Result<Value> {
    let cwd = std::env::current_dir().context("failed to resolve current directory")?;
    let requested_paths = args
        .paths
        .iter()
        .map(|path| display_path(path, &cwd))
        .collect::<Vec<_>>();
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let files = collect_files(&args.paths, &cwd, &mut errors);

    if files.is_empty() && errors.is_empty() {
        push_unique(&mut warnings, "no files matched syntax path scope");
    }

    let options = SyntaxParseOptions {
        max_bytes: args.max_bytes,
        skip_generated_vendor: !args.include_generated_vendor,
    };
    let mut receipts = Vec::new();
    for path in files {
        let display = display_path(&path, &cwd);
        let source = match std::fs::read_to_string(&path) {
            Ok(source) => source,
            Err(err) => {
                push_unique(
                    &mut errors,
                    &format!("failed to read syntax input {display}: {err}"),
                );
                continue;
            }
        };

        let receipt = parse_syntax_receipt(&display, &source, options);
        push_advisory_warning(&receipt, &mut warnings);
        receipts.push(receipt.to_value());
    }
    receipts.sort_by(|left, right| {
        left["path"]
            .as_str()
            .cmp(&right["path"].as_str())
            .then_with(|| left["status"].as_str().cmp(&right["status"].as_str()))
    });

    let status = if !errors.is_empty() {
        "failed"
    } else if !warnings.is_empty() {
        "partial"
    } else {
        "complete"
    };

    Ok(json!({
        "schema": SYNTAX_RECEIPTS_SCHEMA,
        "status": status,
        "paths": requested_paths,
        "max_bytes": args.max_bytes,
        "skip_generated_vendor": !args.include_generated_vendor,
        "receipts": receipts,
        "warnings": warnings,
        "errors": errors,
        "non_claims": [
            "syntax receipts package advisory parser evidence; they do not prove reachability, bug presence, UB presence, safety, or merge readiness"
        ],
    }))
}

fn collect_files(paths: &[PathBuf], cwd: &Path, errors: &mut Vec<String>) -> Vec<PathBuf> {
    let mut files = BTreeSet::new();
    for path in paths {
        if path.is_file() {
            files.insert(path.clone());
        } else if path.is_dir() {
            match tokmd_scan::walk::list_files(path, None) {
                Ok(rel_paths) => {
                    for rel in rel_paths {
                        files.insert(path.join(rel));
                    }
                }
                Err(err) => {
                    push_unique(
                        errors,
                        &format!(
                            "failed to list syntax input {}: {err}",
                            display_path(path, cwd)
                        ),
                    );
                }
            }
        } else {
            push_unique(
                errors,
                &format!("syntax input missing: {}", display_path(path, cwd)),
            );
        }
    }
    files.into_iter().collect()
}

fn push_advisory_warning(receipt: &SyntaxParseReceipt, warnings: &mut Vec<String>) {
    if receipt.status.is_advisory() {
        let reason = receipt.reason.as_deref().unwrap_or("no reason recorded");
        push_unique(
            warnings,
            &format!("{}: {}: {reason}", receipt.path, receipt.status.as_str()),
        );
    }
}

fn display_path(path: &Path, cwd: &Path) -> String {
    let rel = if path.is_absolute() {
        path.strip_prefix(cwd).unwrap_or(path)
    } else {
        path
    };
    tokmd_scan::normalize_slashes(&rel.display().to_string())
        .trim_start_matches("./")
        .to_owned()
}

fn push_unique(values: &mut Vec<String>, value: &str) {
    if values.iter().all(|existing| existing != value) {
        values.push(value.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_path_normalizes_relative_separators() {
        assert_eq!(
            display_path(Path::new(".\\src\\main.rs"), Path::new(".")),
            "src/main.rs"
        );
    }
}
