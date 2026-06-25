//! Evidence packet manifest writer.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde_json::Value;
use tokmd_types::{
    EVIDENCE_PACKET_SCHEMA, EvidencePacketArtifacts, EvidencePacketManifest,
    EvidencePacketReviewPriorityItem, EvidencePacketStatus,
};

use crate::cli;

const MAX_REVIEW_PRIORITY_ITEMS: usize = 10;

struct PacketArtifactPaths<'a> {
    analyze_md: &'a Path,
    analyze_json: &'a Path,
    context_md: &'a Path,
    syntax_json: Option<&'a Path>,
}

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
    let default_syntax_json = output_dir.join("syntax.json");
    let syntax_json = args
        .syntax_json
        .clone()
        .or_else(|| default_syntax_json.is_file().then_some(default_syntax_json));

    let mut warnings = Vec::new();
    let mut errors = Vec::new();
    let mut review_priority = Vec::new();
    let paths = normalize_paths(&args.paths);

    validate_refs(&cwd, &args.base, &args.head, &mut errors);
    require_artifact("analyze_md", &analyze_md, &mut errors);
    require_artifact("analyze_json", &analyze_json, &mut errors);
    require_artifact("context_md", &context_md, &mut errors);
    if let Some(path) = &syntax_json {
        optional_artifact("syntax_json", path, &mut warnings);
    }

    if analyze_json.is_file() {
        inspect_analyze_json(&analyze_json, preset, &paths, &mut warnings, &mut errors);
    }
    if let Some(path) = &syntax_json
        && path.is_file()
    {
        inspect_syntax_json(
            path,
            &paths,
            &manifest_path(path, &cwd),
            &mut warnings,
            &mut review_priority,
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
        syntax_json: syntax_json.as_ref().map(|path| manifest_path(path, &cwd)),
    };
    let review_priority = rank_review_priority(review_priority);

    Ok(EvidencePacketManifest {
        schema: EVIDENCE_PACKET_SCHEMA.to_string(),
        tokmd_version: env!("CARGO_PKG_VERSION").to_string(),
        preset: preset.to_string(),
        base: args.base.clone(),
        head: args.head.clone(),
        paths: paths.clone(),
        status,
        artifacts,
        review_priority,
        warnings,
        errors,
        non_claims: non_claims_for_preset(preset),
        reproduce: reproduce_commands(
            args,
            preset,
            &paths,
            PacketArtifactPaths {
                analyze_md: &analyze_md,
                analyze_json: &analyze_json,
                context_md: &context_md,
                syntax_json: syntax_json.as_deref(),
            },
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

fn optional_artifact(label: &str, path: &Path, warnings: &mut Vec<String>) {
    if !path.is_file() {
        push_unique(
            warnings,
            &format!("optional artifact {label} missing: {}", display_path(path)),
        );
    }
}

fn read_utf8_text(label: &str, path: &Path) -> Result<String, String> {
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(err) => {
            return Err(format!(
                "failed to read {label} {}: {err}",
                display_path(path)
            ));
        }
    };
    if bytes.starts_with(&[0xFF, 0xFE]) {
        return Err(format!(
            "{label} {} appears to be UTF-16LE (PowerShell `>` redirect writes UTF-16 on Windows); \
             prefer tokmd --output-dir / --output, or Out-File -Encoding utf8; \
             see docs/evidence-packet.md#windows-powershell",
            display_path(path)
        ));
    }
    if bytes.starts_with(&[0xFE, 0xFF]) {
        return Err(format!(
            "{label} {} appears to be UTF-16BE; tokmd JSON inputs must be UTF-8; \
             see docs/evidence-packet.md#windows-powershell",
            display_path(path)
        ));
    }
    match std::str::from_utf8(&bytes) {
        Ok(text) => Ok(text.to_string()),
        Err(_) => Err(format!(
            "{label} {} is not valid UTF-8; on Windows PowerShell, bare `>` redirect writes UTF-16LE — \
             use tokmd --output-dir / --output or Out-File -Encoding utf8; \
             see docs/evidence-packet.md#windows-powershell",
            display_path(path)
        )),
    }
}

fn inspect_analyze_json(
    path: &Path,
    expected_preset: &str,
    expected_paths: &[String],
    warnings: &mut Vec<String>,
    errors: &mut Vec<String>,
) {
    let content = match read_utf8_text("analyze_json", path) {
        Ok(content) => content,
        Err(message) => {
            push_unique(errors, &message);
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

fn inspect_syntax_json(
    path: &Path,
    expected_paths: &[String],
    artifact_ref: &str,
    warnings: &mut Vec<String>,
    review_priority: &mut Vec<EvidencePacketReviewPriorityItem>,
) {
    let content = match read_utf8_text("syntax_json", path) {
        Ok(content) => content,
        Err(message) => {
            push_unique(warnings, &message);
            return;
        }
    };
    let json: Value = match serde_json::from_str(&content) {
        Ok(json) => json,
        Err(err) => {
            push_unique(
                warnings,
                &format!("failed to parse syntax_json {}: {err}", display_path(path)),
            );
            return;
        }
    };

    match json.get("schema").and_then(Value::as_str) {
        Some("tokmd.syntax_receipts.v1") => {}
        Some(other) => push_unique(
            warnings,
            &format!("syntax_json has unsupported schema '{other}'"),
        ),
        None => push_unique(warnings, "syntax_json is missing schema"),
    }

    match json.get("status").and_then(Value::as_str) {
        Some("complete") => {}
        Some("partial") => push_unique(warnings, "syntax_json status is partial"),
        Some("failed") => push_unique(warnings, "syntax_json status is failed"),
        Some(other) => push_unique(
            warnings,
            &format!("syntax_json has unsupported status '{other}'"),
        ),
        None => push_unique(warnings, "syntax_json is missing status"),
    }

    match json.get("paths").and_then(Value::as_array) {
        Some(inputs) => {
            let actual: Vec<String> = inputs
                .iter()
                .filter_map(Value::as_str)
                .map(normalize_manifest_path)
                .collect();
            if actual.len() != inputs.len() {
                push_unique(warnings, "syntax_json paths contains non-string values");
            } else if actual != expected_paths {
                push_unique(
                    warnings,
                    &format!(
                        "syntax_json paths {:?} do not match requested paths {:?}",
                        actual, expected_paths
                    ),
                );
            }
        }
        None => push_unique(warnings, "syntax_json is missing paths"),
    }

    match json.get("warnings").and_then(Value::as_array) {
        Some(items) => {
            for item in items {
                match item.as_str() {
                    Some(warning) if !warning.is_empty() => {
                        push_unique(warnings, &format!("syntax_json warning: {warning}"))
                    }
                    Some(_) => {}
                    None => {
                        push_unique(warnings, "syntax_json warnings contains non-string values")
                    }
                }
            }
        }
        None => push_unique(warnings, "syntax_json is missing warnings"),
    }

    match json.get("errors").and_then(Value::as_array) {
        Some(items) => {
            for item in items {
                match item.as_str() {
                    Some(error) if !error.is_empty() => {
                        push_unique(warnings, &format!("syntax_json error: {error}"))
                    }
                    Some(_) => {}
                    None => push_unique(warnings, "syntax_json errors contains non-string values"),
                }
            }
        }
        None => push_unique(warnings, "syntax_json is missing errors"),
    }

    collect_syntax_review_priority(&json, artifact_ref, warnings, review_priority);
}

fn collect_syntax_review_priority(
    json: &Value,
    artifact_ref: &str,
    warnings: &mut Vec<String>,
    review_priority: &mut Vec<EvidencePacketReviewPriorityItem>,
) {
    let receipts = match json.get("receipts").and_then(Value::as_array) {
        Some(receipts) => receipts,
        None => {
            push_unique(warnings, "syntax_json is missing receipts");
            return;
        }
    };

    for (receipt_index, receipt) in receipts.iter().enumerate() {
        let Some(path) = receipt.get("path").and_then(Value::as_str) else {
            push_unique(
                warnings,
                &format!("syntax_json receipt {receipt_index} is missing path"),
            );
            continue;
        };
        let Some(signals) = receipt.get("review_signals").and_then(Value::as_array) else {
            push_unique(
                warnings,
                &format!("syntax_json receipt {receipt_index} is missing review_signals"),
            );
            continue;
        };

        for (signal_index, signal) in signals.iter().enumerate() {
            let Some(category) =
                syntax_signal_string(signal, "category", receipt_index, signal_index, warnings)
            else {
                continue;
            };
            let Some(severity) =
                syntax_signal_string(signal, "severity", receipt_index, signal_index, warnings)
            else {
                continue;
            };
            let Some(kind) =
                syntax_signal_string(signal, "kind", receipt_index, signal_index, warnings)
            else {
                continue;
            };
            let Some(reason) =
                syntax_signal_string(signal, "reason", receipt_index, signal_index, warnings)
            else {
                continue;
            };
            let Some(evidence) =
                syntax_signal_string(signal, "evidence", receipt_index, signal_index, warnings)
            else {
                continue;
            };
            let Some(score) = syntax_signal_score(signal, receipt_index, signal_index, warnings)
            else {
                continue;
            };

            review_priority.push(EvidencePacketReviewPriorityItem {
                rank: 0,
                path: normalize_manifest_path(path),
                category,
                severity,
                score,
                kind,
                reason,
                evidence,
                refs: vec![format!(
                    "{artifact_ref}#/receipts/{receipt_index}/review_signals/{signal_index}"
                )],
            });
        }
    }
}

fn syntax_signal_string(
    signal: &Value,
    field: &str,
    receipt_index: usize,
    signal_index: usize,
    warnings: &mut Vec<String>,
) -> Option<String> {
    match signal.get(field).and_then(Value::as_str) {
        Some(value) if !value.is_empty() => Some(value.to_string()),
        Some(_) => {
            push_unique(
                warnings,
                &format!(
                    "syntax_json receipt {receipt_index} review_signals {signal_index} has empty {field}"
                ),
            );
            None
        }
        None => {
            push_unique(
                warnings,
                &format!(
                    "syntax_json receipt {receipt_index} review_signals {signal_index} is missing {field}"
                ),
            );
            None
        }
    }
}

fn syntax_signal_score(
    signal: &Value,
    receipt_index: usize,
    signal_index: usize,
    warnings: &mut Vec<String>,
) -> Option<u32> {
    let Some(score) = signal.get("score").and_then(Value::as_u64) else {
        push_unique(
            warnings,
            &format!(
                "syntax_json receipt {receipt_index} review_signals {signal_index} is missing score"
            ),
        );
        return None;
    };
    Some(u32::try_from(score).unwrap_or(u32::MAX))
}

fn rank_review_priority(
    mut items: Vec<EvidencePacketReviewPriorityItem>,
) -> Vec<EvidencePacketReviewPriorityItem> {
    items.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| severity_rank(&a.severity).cmp(&severity_rank(&b.severity)))
            .then_with(|| a.path.cmp(&b.path))
            .then_with(|| a.category.cmp(&b.category))
            .then_with(|| a.kind.cmp(&b.kind))
            .then_with(|| a.evidence.cmp(&b.evidence))
    });
    items.truncate(MAX_REVIEW_PRIORITY_ITEMS);
    for (index, item) in items.iter_mut().enumerate() {
        item.rank = u32::try_from(index + 1).unwrap_or(u32::MAX);
    }
    items
}

fn severity_rank(severity: &str) -> u8 {
    match severity {
        "high" => 0,
        "medium" => 1,
        "low" => 2,
        _ => 3,
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
    artifact_paths: PacketArtifactPaths<'_>,
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
            quote_arg(&manifest_path(artifact_paths.analyze_md, cwd))
        ));
    }
    if args.analyze_json.is_some() {
        packet_command.push_str(&format!(
            " --analyze-json {}",
            quote_arg(&manifest_path(artifact_paths.analyze_json, cwd))
        ));
    }
    if args.context_md.is_some() {
        packet_command.push_str(&format!(
            " --context-md {}",
            quote_arg(&manifest_path(artifact_paths.context_md, cwd))
        ));
    }
    if let Some(path) = artifact_paths.syntax_json
        && args.syntax_json.is_some()
    {
        packet_command.push_str(&format!(
            " --syntax-json {}",
            quote_arg(&manifest_path(path, cwd))
        ));
    }
    packet_command.push(' ');
    packet_command.push_str(&joined_paths);

    let sensor_dir = manifest_path(
        artifact_paths
            .analyze_json
            .parent()
            .unwrap_or_else(|| Path::new(".")),
        cwd,
    );
    let mut commands = vec![
        format!(
            "tokmd analyze --preset {preset} --format md --effort-base-ref {} --effort-head-ref {} --no-progress --output-dir {} {joined_paths}",
            quote_arg(&args.base),
            quote_arg(&args.head),
            quote_arg(&sensor_dir),
        ),
        format!(
            "tokmd analyze --preset {preset} --format json --effort-base-ref {} --effort-head-ref {} --no-progress --output-dir {} {joined_paths}",
            quote_arg(&args.base),
            quote_arg(&args.head),
            quote_arg(&sensor_dir),
        ),
        format!(
            "tokmd context --budget {} --output {} {joined_paths}",
            quote_arg(&args.context_budget),
            quote_arg(&manifest_path(artifact_paths.context_md, cwd)),
        ),
    ];
    if let Some(path) = artifact_paths.syntax_json {
        commands.push(format!(
            "tokmd syntax --no-progress {joined_paths} > {}",
            quote_arg(&manifest_path(path, cwd)),
        ));
    }
    commands.push(packet_command);
    commands
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

    #[test]
    fn read_utf8_text_rejects_utf16le_bom() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("analyze.json");
        std::fs::write(&path, [0xFF_u8, 0xFE, b'{', b'}']).unwrap();

        let err = read_utf8_text("analyze_json", &path).unwrap_err();
        assert!(err.contains("UTF-16LE"));
        assert!(err.contains("PowerShell"));
    }

    #[test]
    fn read_utf8_text_accepts_utf8_json() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("analyze.json");
        std::fs::write(&path, r#"{"status":"complete"}"#).unwrap();

        let text = read_utf8_text("analyze_json", &path).unwrap();
        assert!(text.contains("complete"));
    }
}
