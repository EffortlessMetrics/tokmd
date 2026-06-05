//! Evidence packet manifest writer.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde_json::Value;
use tokmd_types::{
    EVIDENCE_PACKET_SCHEMA, EvidencePacketArtifacts, EvidencePacketManifest, EvidencePacketStatus,
};

use crate::cli;

pub(crate) fn handle(args: cli::EvidencePacketArgs) -> Result<()> {
    let manifest = build_manifest(&args)?;
    write_manifest(&args.output, &manifest)?;

    let json = serde_json::to_string_pretty(&manifest)?;
    println!("{json}");

    if manifest.status == EvidencePacketStatus::Failed {
        let detail = if manifest.errors.is_empty() {
            "unknown error".to_string()
        } else {
            manifest.errors.join("; ")
        };
        bail!("evidence packet failed: {detail}");
    }

    Ok(())
}

fn build_manifest(args: &cli::EvidencePacketArgs) -> Result<EvidencePacketManifest> {
    let cwd = std::env::current_dir().context("failed to resolve current directory")?;
    let preset = preset_to_string(args.preset);
    let output_dir = args.output.parent().unwrap_or_else(|| Path::new("."));
    let analyze_md = args
        .analyze_md
        .clone()
        .unwrap_or_else(|| output_dir.join("analyze.md"));
    let analyze_json = args
        .analyze_json
        .clone()
        .unwrap_or_else(|| output_dir.join("analyze.json"));
    let context_md = args
        .context_md
        .clone()
        .unwrap_or_else(|| output_dir.join("context.md"));

    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    validate_refs(&cwd, &args.base, &args.head, &mut errors);
    require_artifact("analyze_md", &analyze_md, &mut errors);
    require_artifact("analyze_json", &analyze_json, &mut errors);
    require_artifact("context_md", &context_md, &mut errors);

    if analyze_json.is_file() {
        inspect_analyze_json(
            &analyze_json,
            preset,
            &normalize_paths(&args.paths),
            &mut warnings,
            &mut errors,
        );
    }

    let status = if !errors.is_empty() {
        EvidencePacketStatus::Failed
    } else if !warnings.is_empty() {
        EvidencePacketStatus::Partial
    } else {
        EvidencePacketStatus::Complete
    };

    let artifacts = EvidencePacketArtifacts {
        analyze_md: manifest_path(&analyze_md, &cwd),
        analyze_json: manifest_path(&analyze_json, &cwd),
        context_md: manifest_path(&context_md, &cwd),
    };
    let paths = normalize_paths(&args.paths);

    Ok(EvidencePacketManifest {
        schema: EVIDENCE_PACKET_SCHEMA.to_string(),
        tokmd_version: env!("CARGO_PKG_VERSION").to_string(),
        preset: preset.to_string(),
        base: args.base.clone(),
        head: args.head.clone(),
        paths: paths.clone(),
        status,
        artifacts,
        warnings,
        errors,
        non_claims: non_claims_for_preset(preset),
        reproduce: reproduce_commands(
            args,
            preset,
            &paths,
            &analyze_md,
            &analyze_json,
            &context_md,
            &cwd,
        ),
    })
}

#[cfg(feature = "git")]
fn validate_refs(cwd: &Path, base: &str, head: &str, errors: &mut Vec<String>) {
    if !tokmd_git::git_available() {
        push_unique(errors, "git is not available on PATH");
        return;
    }
    let Some(repo_root) = tokmd_git::repo_root(cwd) else {
        push_unique(
            errors,
            "failed to locate git repository for base/head validation",
        );
        return;
    };
    for rev in [base, head] {
        if !tokmd_git::rev_exists(&repo_root, rev) {
            push_unique(errors, &format!("could not resolve ref '{rev}'"));
        }
    }
}

#[cfg(not(feature = "git"))]
fn validate_refs(_cwd: &Path, _base: &str, _head: &str, errors: &mut Vec<String>) {
    push_unique(
        errors,
        "base/head validation requires the tokmd git feature",
    );
}

fn require_artifact(label: &str, path: &Path, errors: &mut Vec<String>) {
    if !path.is_file() {
        push_unique(
            errors,
            &format!("required artifact {label} missing: {}", display_path(path)),
        );
    }
}

fn inspect_analyze_json(
    path: &Path,
    expected_preset: &str,
    expected_paths: &[String],
    warnings: &mut Vec<String>,
    errors: &mut Vec<String>,
) {
    let content = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(err) => {
            push_unique(
                errors,
                &format!("failed to read analyze_json {}: {err}", display_path(path)),
            );
            return;
        }
    };
    let json: Value = match serde_json::from_str(&content) {
        Ok(json) => json,
        Err(err) => {
            push_unique(
                errors,
                &format!("failed to parse analyze_json {}: {err}", display_path(path)),
            );
            return;
        }
    };

    match json.get("status").and_then(Value::as_str) {
        Some("complete") => {}
        Some("partial") => push_unique(warnings, "analyze.json status is partial"),
        Some(other) => push_unique(
            errors,
            &format!("analyze.json has unsupported status '{other}'"),
        ),
        None => push_unique(errors, "analyze.json is missing status"),
    }

    let actual_preset = json
        .pointer("/args/preset")
        .or_else(|| json.get("preset"))
        .and_then(Value::as_str);
    match actual_preset {
        Some(actual) if actual == expected_preset => {}
        Some(actual) => push_unique(
            errors,
            &format!(
                "analyze.json preset '{actual}' does not match requested preset '{expected_preset}'"
            ),
        ),
        None => push_unique(errors, "analyze.json is missing args.preset"),
    }

    match json.pointer("/source/inputs").and_then(Value::as_array) {
        Some(inputs) => {
            let actual: Vec<String> = inputs
                .iter()
                .filter_map(Value::as_str)
                .map(normalize_manifest_path)
                .collect();
            if actual.len() != inputs.len() {
                push_unique(
                    errors,
                    "analyze.json source.inputs contains non-string values",
                );
            } else if actual != expected_paths {
                push_unique(
                    errors,
                    &format!(
                        "analyze.json source.inputs {:?} do not match requested paths {:?}",
                        actual, expected_paths
                    ),
                );
            }
        }
        None => push_unique(errors, "analyze.json is missing source.inputs"),
    }

    match json.get("warnings").and_then(Value::as_array) {
        Some(items) => {
            for item in items {
                match item.as_str() {
                    Some(warning) if !warning.is_empty() => push_unique(warnings, warning),
                    Some(_) => {}
                    None => push_unique(errors, "analyze.json warnings contains non-string values"),
                }
            }
        }
        None => push_unique(errors, "analyze.json is missing warnings"),
    }
}

fn write_manifest(path: &Path, manifest: &EvidencePacketManifest) -> Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let mut json = serde_json::to_string_pretty(manifest)?;
    json.push('\n');
    std::fs::write(path, json.as_bytes())
        .with_context(|| format!("failed to write {}", path.display()))
}

fn preset_to_string(preset: cli::AnalysisPreset) -> &'static str {
    match preset {
        cli::AnalysisPreset::Receipt => "receipt",
        cli::AnalysisPreset::Estimate => "estimate",
        cli::AnalysisPreset::BunUb => "bun-ub",
        cli::AnalysisPreset::Health => "health",
        cli::AnalysisPreset::Risk => "risk",
        cli::AnalysisPreset::Supply => "supply",
        cli::AnalysisPreset::Architecture => "architecture",
        cli::AnalysisPreset::Topics => "topics",
        cli::AnalysisPreset::Security => "security",
        cli::AnalysisPreset::Identity => "identity",
        cli::AnalysisPreset::Git => "git",
        cli::AnalysisPreset::Deep => "deep",
        cli::AnalysisPreset::Fun => "fun",
    }
}

fn non_claims_for_preset(preset: &str) -> Vec<String> {
    if preset == "bun-ub" {
        vec![
            "bun-ub packages review evidence; it does not prove UB exists or is absent".to_string(),
        ]
    } else {
        vec![
            "tokmd evidence packets package scoped review evidence; they do not prove safety, correctness, or merge readiness"
                .to_string(),
        ]
    }
}

fn reproduce_commands(
    args: &cli::EvidencePacketArgs,
    preset: &str,
    paths: &[String],
    analyze_md: &Path,
    analyze_json: &Path,
    context_md: &Path,
    cwd: &Path,
) -> Vec<String> {
    let joined_paths = paths
        .iter()
        .map(|path| quote_arg(path))
        .collect::<Vec<_>>()
        .join(" ");
    let manifest_output = manifest_path(&args.output, cwd);
    let mut packet_command = format!(
        "tokmd evidence-packet --preset {preset} --base {} --head {} --output {} --context-budget {}",
        quote_arg(&args.base),
        quote_arg(&args.head),
        quote_arg(&manifest_output),
        quote_arg(&args.context_budget),
    );
    if args.analyze_md.is_some() {
        packet_command.push_str(&format!(
            " --analyze-md {}",
            quote_arg(&manifest_path(analyze_md, cwd))
        ));
    }
    if args.analyze_json.is_some() {
        packet_command.push_str(&format!(
            " --analyze-json {}",
            quote_arg(&manifest_path(analyze_json, cwd))
        ));
    }
    if args.context_md.is_some() {
        packet_command.push_str(&format!(
            " --context-md {}",
            quote_arg(&manifest_path(context_md, cwd))
        ));
    }
    packet_command.push(' ');
    packet_command.push_str(&joined_paths);

    vec![
        format!(
            "tokmd analyze --preset {preset} --format md --effort-base-ref {} --effort-head-ref {} --no-progress {joined_paths} > {}",
            quote_arg(&args.base),
            quote_arg(&args.head),
            quote_arg(&manifest_path(analyze_md, cwd)),
        ),
        format!(
            "tokmd analyze --preset {preset} --format json --effort-base-ref {} --effort-head-ref {} --no-progress {joined_paths} > {}",
            quote_arg(&args.base),
            quote_arg(&args.head),
            quote_arg(&manifest_path(analyze_json, cwd)),
        ),
        format!(
            "tokmd context --budget {} {joined_paths} > {}",
            quote_arg(&args.context_budget),
            quote_arg(&manifest_path(context_md, cwd)),
        ),
        packet_command,
    ]
}

fn normalize_paths(paths: &[PathBuf]) -> Vec<String> {
    paths
        .iter()
        .map(|path| normalize_manifest_path(&path.display().to_string()))
        .collect()
}

fn normalize_manifest_path(path: &str) -> String {
    let normalized = tokmd_scan::normalize_slashes(path);
    normalized.trim_start_matches("./").to_string()
}

fn manifest_path(path: &Path, cwd: &Path) -> String {
    let rel = if path.is_absolute() {
        path.strip_prefix(cwd).unwrap_or(path)
    } else {
        path
    };
    normalize_manifest_path(&rel.display().to_string())
}

fn display_path(path: &Path) -> String {
    normalize_manifest_path(&path.display().to_string())
}

fn quote_arg(value: &str) -> String {
    if value.is_empty()
        || value.chars().any(|ch| {
            ch.is_whitespace()
                || matches!(
                    ch,
                    '"' | '\'' | '$' | '`' | '&' | '|' | '<' | '>' | ';' | '(' | ')'
                )
        })
    {
        format!("\"{}\"", value.replace('"', "\\\""))
    } else {
        value.to_string()
    }
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
    fn quote_arg_leaves_simple_paths_unquoted() {
        assert_eq!(quote_arg("src/runtime/api"), "src/runtime/api");
    }

    #[test]
    fn quote_arg_quotes_whitespace() {
        assert_eq!(quote_arg("src/runtime api"), "\"src/runtime api\"");
    }

    #[test]
    fn normalize_manifest_path_uses_forward_slashes() {
        assert_eq!(
            normalize_manifest_path(".\\src\\runtime\\api"),
            "src/runtime/api"
        );
    }
}
