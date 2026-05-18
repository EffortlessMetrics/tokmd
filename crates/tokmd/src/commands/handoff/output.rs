//! Handoff bundle output writers.

use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use blake3::Hasher;
use serde_json::{Value, json};
use tokmd_types::{
    ArtifactEntry, ArtifactHash, ContextFileRow, ExportData, FileKind, HandoffIntelligence,
    HandoffManifest, InclusionPolicy,
};

mod linked_evidence;

use linked_evidence::LinkedEvidenceSummary;

pub(super) struct HandoffPayloads {
    pub(super) map_bytes: u64,
    pub(super) intelligence_bytes: u64,
    pub(super) code_bytes: u64,
    pub(super) artifacts: Vec<ArtifactEntry>,
}

pub(super) struct HandoffLinkInputs<'a> {
    pub(super) review_packet_dir: Option<&'a Path>,
    pub(super) review_packet_check: Option<&'a Path>,
    pub(super) affected: Option<&'a Path>,
    pub(super) proof_plan: Option<&'a Path>,
}

pub(super) struct HandoffWorkOrderInputs<'a> {
    pub(super) inputs: &'a [String],
    pub(super) budget_tokens: usize,
    pub(super) used_tokens: usize,
    pub(super) utilization_pct: f64,
    pub(super) strategy: &'a str,
    pub(super) rank_by: &'a str,
    pub(super) intelligence_preset: &'a str,
    pub(super) total_files: usize,
    pub(super) selected: &'a [ContextFileRow],
    pub(super) links: &'a HandoffLinkInputs<'a>,
}

pub(super) fn write_payloads(
    out_dir: &Path,
    export: &ExportData,
    intelligence: &HandoffIntelligence,
    selected: &[ContextFileRow],
    compress: bool,
) -> Result<HandoffPayloads> {
    let map_path = out_dir.join("map.jsonl");
    let map_bytes = write_map_jsonl(&map_path, export)?;
    let map_hash = hash_file(&map_path)?;

    let intelligence_path = out_dir.join("intelligence.json");
    let intelligence_json = serde_json::to_string_pretty(intelligence)?;
    fs::write(&intelligence_path, &intelligence_json)
        .with_context(|| format!("Failed to write {}", intelligence_path.display()))?;
    let intelligence_bytes = intelligence_json.len() as u64;
    let intelligence_hash = hash_bytes(intelligence_json.as_bytes());

    let code_path = out_dir.join("code.txt");
    let code_bytes = write_code_bundle(&code_path, selected, compress)?;
    let code_hash = hash_file(&code_path)?;

    let artifacts = vec![
        ArtifactEntry {
            name: "manifest".to_string(),
            path: "manifest.json".to_string(),
            description: "Bundle metadata and capabilities".to_string(),
            bytes: 0,
            hash: None,
        },
        ArtifactEntry {
            name: "map".to_string(),
            path: "map.jsonl".to_string(),
            description: "Complete file inventory".to_string(),
            bytes: map_bytes,
            hash: Some(ArtifactHash {
                algo: "blake3".to_string(),
                hash: map_hash,
            }),
        },
        ArtifactEntry {
            name: "intelligence".to_string(),
            path: "intelligence.json".to_string(),
            description: "Tree, hotspots, complexity, and derived metrics".to_string(),
            bytes: intelligence_bytes,
            hash: Some(ArtifactHash {
                algo: "blake3".to_string(),
                hash: intelligence_hash,
            }),
        },
        ArtifactEntry {
            name: "code".to_string(),
            path: "code.txt".to_string(),
            description: "Token-budgeted code bundle".to_string(),
            bytes: code_bytes,
            hash: Some(ArtifactHash {
                algo: "blake3".to_string(),
                hash: code_hash,
            }),
        },
    ];

    Ok(HandoffPayloads {
        map_bytes,
        intelligence_bytes,
        code_bytes,
        artifacts,
    })
}

pub(super) fn write_link_artifacts(
    out_dir: &Path,
    links: &HandoffLinkInputs<'_>,
) -> Result<Vec<ArtifactEntry>> {
    let mut artifacts = Vec::new();

    if links.review_packet_dir.is_some() || links.review_packet_check.is_some() {
        artifacts.push(write_json_artifact(
            out_dir,
            "review-links",
            "review-links.json",
            "Linked cockpit review packet artifacts",
            &review_links_json(links.review_packet_dir, links.review_packet_check),
        )?);
    }

    if links.affected.is_some() || links.proof_plan.is_some() {
        artifacts.push(write_json_artifact(
            out_dir,
            "proof-links",
            "proof-links.json",
            "Linked affected-proof and proof-plan artifacts",
            &proof_links_json(links.affected, links.proof_plan),
        )?);
    }

    Ok(artifacts)
}

pub(super) fn write_work_order(
    out_dir: &Path,
    order: &HandoffWorkOrderInputs<'_>,
) -> Result<ArtifactEntry> {
    let linked_evidence = linked_evidence::summarize(order.links);
    write_text_artifact(
        out_dir,
        "work-order",
        "work-order.md",
        "Agent work order and consumption guide",
        &render_work_order(order, &linked_evidence),
    )
}

pub(super) fn write_manifest_json(out_dir: &Path, manifest: &HandoffManifest) -> Result<usize> {
    let manifest_path = out_dir.join("manifest.json");
    let manifest_json = serde_json::to_string_pretty(manifest)?;
    fs::write(&manifest_path, &manifest_json)
        .with_context(|| format!("Failed to write {}", manifest_path.display()))?;
    Ok(manifest_json.len())
}

fn write_json_artifact(
    out_dir: &Path,
    name: &str,
    relative_path: &str,
    description: &str,
    value: &Value,
) -> Result<ArtifactEntry> {
    let path = out_dir.join(relative_path);
    let json = serde_json::to_string_pretty(value)?;
    fs::write(&path, &json).with_context(|| format!("Failed to write {}", path.display()))?;

    Ok(ArtifactEntry {
        name: name.to_string(),
        path: relative_path.to_string(),
        description: description.to_string(),
        bytes: json.len() as u64,
        hash: Some(ArtifactHash {
            algo: "blake3".to_string(),
            hash: hash_bytes(json.as_bytes()),
        }),
    })
}

fn write_text_artifact(
    out_dir: &Path,
    name: &str,
    relative_path: &str,
    description: &str,
    content: &str,
) -> Result<ArtifactEntry> {
    let path = out_dir.join(relative_path);
    fs::write(&path, content).with_context(|| format!("Failed to write {}", path.display()))?;

    Ok(ArtifactEntry {
        name: name.to_string(),
        path: relative_path.to_string(),
        description: description.to_string(),
        bytes: content.len() as u64,
        hash: Some(ArtifactHash {
            algo: "blake3".to_string(),
            hash: hash_bytes(content.as_bytes()),
        }),
    })
}

fn render_work_order(
    order: &HandoffWorkOrderInputs<'_>,
    linked_evidence: &LinkedEvidenceSummary,
) -> String {
    let mut out = String::new();
    push_work_order_header(&mut out);
    push_start_here_section(&mut out, order.links);
    push_bundle_summary_section(&mut out, order);
    push_changed_surfaces_section(&mut out, order.links, linked_evidence);
    push_linked_evidence_section(&mut out, order.links);
    linked_evidence::render(&mut out, order.links, linked_evidence);
    push_review_evidence_section(&mut out, order.links, linked_evidence);
    push_proof_expectations_section(&mut out, order.links, linked_evidence);
    push_missing_evidence_section(&mut out, linked_evidence);
    push_included_files_section(&mut out, order.selected);
    push_agent_stop_conditions_section(&mut out, order.links, linked_evidence);
    push_agent_guardrails_section(&mut out);
    out
}

fn push_work_order_header(out: &mut String) {
    out.push_str("# Agent Work Order\n\n");
    out.push_str("This handoff is a deterministic source/context bundle for coding-agent work.\n");
    out.push_str("Treat linked review and proof receipts as external evidence handles; this file does not verify them.\n\n");
}

fn push_start_here_section(out: &mut String, links: &HandoffLinkInputs<'_>) {
    out.push_str("## Start Here\n\n");
    let mut steps = vec![
        "Read `manifest.json` for the authoritative artifact index, token budget, included files, and exclusions.",
        "Read `work-order.md` for the agent task map and guardrails.",
        "Read `code.txt` for the bounded source bundle.",
        "Use `map.jsonl` for full file inventory and path lookup.",
        "Use `intelligence.json` for repository shape, hotspots, complexity, and derived signals.",
    ];
    if links.review_packet_dir.is_some() || links.review_packet_check.is_some() {
        steps.push(
            "Use `review-links.json` for cockpit review packet and verifier receipt pointers.",
        );
    }
    if links.affected.is_some() || links.proof_plan.is_some() {
        steps.push("Use `proof-links.json` for affected-proof and proof-plan pointers.");
    }
    for (index, step) in steps.iter().enumerate() {
        out.push_str(&format!("{}. {}\n", index + 1, step));
    }
}

fn push_bundle_summary_section(out: &mut String, order: &HandoffWorkOrderInputs<'_>) {
    out.push_str("\n## Bundle Summary\n\n");
    out.push_str(&format!("- Inputs: {}\n", order.inputs.join(", ")));
    out.push_str(&format!("- Budget tokens: {}\n", order.budget_tokens));
    out.push_str(&format!("- Used tokens: {}\n", order.used_tokens));
    out.push_str(&format!("- Utilization: {:.2}%\n", order.utilization_pct));
    out.push_str(&format!("- Strategy: `{}`\n", order.strategy));
    out.push_str(&format!("- Rank metric: `{}`\n", order.rank_by));
    out.push_str(&format!(
        "- Intelligence preset: `{}`\n",
        order.intelligence_preset
    ));
    out.push_str(&format!("- Bundled files: {}\n", order.selected.len()));
    out.push_str(&format!("- Total scanned files: {}\n", order.total_files));
}

fn push_changed_surfaces_section(
    out: &mut String,
    links: &HandoffLinkInputs<'_>,
    summary: &LinkedEvidenceSummary,
) {
    out.push_str("\n## Changed Surfaces\n\n");
    if let Some(affected) = &summary.affected {
        out.push_str(&format!(
            "- Affected proof reports {} changed file(s), {} matched scope(s), and {} unknown file(s).\n",
            affected.changed_files, affected.scopes, affected.unknown_files
        ));
        if !affected.scope_names.is_empty() {
            out.push_str("- Matched scopes: ");
            out.push_str(&affected.scope_names.join(", "));
            out.push('\n');
        }
        if !affected.changed_file_paths.is_empty() {
            out.push_str("- Changed files to inspect first:\n");
            for path in &affected.changed_file_paths {
                out.push_str(&format!("  - `{path}`\n"));
            }
            if affected.changed_files > affected.changed_file_paths.len() {
                out.push_str(&format!(
                    "  - ... {} more changed file(s); open the affected report for the full list.\n",
                    affected.changed_files - affected.changed_file_paths.len()
                ));
            }
        }
    } else if links.affected.is_some() {
        out.push_str("- Affected proof report is linked but not readable; regenerate or inspect `proof-links.json`.\n");
    } else {
        out.push_str("- No affected-proof report linked. Treat bundled files as context, not a complete change list.\n");
    }
}

fn push_linked_evidence_section(out: &mut String, links: &HandoffLinkInputs<'_>) {
    out.push_str("\n## Linked Evidence\n\n");
    push_linked_path_line(out, "Review packet directory", links.review_packet_dir);
    push_linked_path_line(
        out,
        "Review packet verifier receipt",
        links.review_packet_check,
    );
    push_linked_path_line(out, "Affected proof report", links.affected);
    push_linked_path_line(out, "Proof plan report", links.proof_plan);
}

fn push_linked_path_line(out: &mut String, label: &str, path: Option<&Path>) {
    match path {
        Some(path) => out.push_str(&format!("- {}: `{}`\n", label, path_string(path))),
        None => out.push_str(&format!("- {}: not linked\n", label)),
    }
}

fn push_review_evidence_section(
    out: &mut String,
    links: &HandoffLinkInputs<'_>,
    summary: &LinkedEvidenceSummary,
) {
    out.push_str("\n## Review Evidence\n\n");
    if links.review_packet_dir.is_none() && links.review_packet_check.is_none() {
        out.push_str("- Review packet: not linked.\n");
        out.push_str("- Open first: `work-order.md`, then `code.txt`.\n");
        return;
    }

    if let Some(check) = &summary.review_packet_check {
        match check.ok {
            Some(true) => out.push_str("- Review packet verifier: linked and ok.\n"),
            Some(false) => out.push_str("- Review packet verifier: linked and failing.\n"),
            None => out.push_str("- Review packet verifier: linked with unknown status.\n"),
        }
    } else if links.review_packet_check.is_some() {
        out.push_str("- Review packet verifier: linked but not readable.\n");
    } else {
        out.push_str("- Review packet verifier: not linked.\n");
    }

    if let Some(review_map) = &summary.review_map {
        out.push_str(&format!(
            "- Review map: linked with {} item(s).\n",
            review_map.item_count
        ));
        if !review_map.first_items.is_empty() {
            out.push_str("- Open first from review packet: `review-map.md`.\n");
        }
    } else if links.review_packet_dir.is_some() {
        out.push_str("- Review map: linked but not readable.\n");
    }

    out.push_str("- Reproduce review evidence with commands listed in the linked review map.\n");
}

fn push_proof_expectations_section(
    out: &mut String,
    links: &HandoffLinkInputs<'_>,
    summary: &LinkedEvidenceSummary,
) {
    out.push_str("\n## Proof Expectations\n\n");
    if links.affected.is_none() && links.proof_plan.is_none() {
        out.push_str("- Affected proof and proof plan: not linked.\n");
        out.push_str("- Do not claim scoped proof coverage from this handoff alone.\n");
        return;
    }

    if let Some(affected) = &summary.affected {
        out.push_str(&format!(
            "- Affected proof: {} changed file(s), {} scope(s), {} unknown file(s).\n",
            affected.changed_files, affected.scopes, affected.unknown_files
        ));
    } else if links.affected.is_some() {
        out.push_str("- Affected proof: linked but not readable.\n");
    }

    if let Some(proof_plan) = &summary.proof_plan {
        out.push_str(&format!(
            "- Proof plan: {} command(s), {} required, {} advisory.\n",
            proof_plan.commands, proof_plan.required, proof_plan.advisory
        ));
        if !proof_plan.first_commands.is_empty() {
            out.push_str("- Run expected proof before claiming done:\n");
            for command in &proof_plan.first_commands {
                out.push_str(&format!("  - `{command}`\n"));
            }
            if proof_plan.commands > proof_plan.first_commands.len() {
                out.push_str(&format!(
                    "  - ... {} more command(s); open the proof plan for the full list.\n",
                    proof_plan.commands - proof_plan.first_commands.len()
                ));
            }
        }
    } else if links.proof_plan.is_some() {
        out.push_str("- Proof plan: linked but not readable.\n");
    }

    out.push_str("- Treat proof plans as expectations until an executed proof receipt exists.\n");
}

fn push_missing_evidence_section(out: &mut String, summary: &LinkedEvidenceSummary) {
    out.push_str("\n## Missing / Stale / Degraded Evidence\n\n");
    let mut emitted = false;

    if let Some(review_map) = &summary.review_map {
        emitted |= push_issue_count(out, "Review evidence missing", review_map.missing);
        emitted |= push_issue_count(out, "Review evidence stale", review_map.stale);
        emitted |= push_issue_count(out, "Review evidence degraded", review_map.degraded);
        emitted |= push_issue_count(out, "Review evidence skipped", review_map.skipped);
        emitted |= push_issue_count(out, "Review evidence unavailable", review_map.unavailable);
    }

    if let Some(affected) = &summary.affected
        && affected.unknown_files > 0
    {
        out.push_str(&format!(
            "- Affected proof has {} unknown file(s); update proof routing before trusting scoped proof.\n",
            affected.unknown_files
        ));
        emitted = true;
    }

    if !emitted {
        out.push_str("- No missing, stale, degraded, skipped, unavailable, or unknown-file evidence was reported by linked summaries.\n");
    }
}

fn push_issue_count(out: &mut String, label: &str, count: Option<u64>) -> bool {
    let Some(count) = count else {
        return false;
    };
    if count == 0 {
        return false;
    }
    out.push_str(&format!("- {label}: {count}\n"));
    true
}

fn push_included_files_section(out: &mut String, selected: &[ContextFileRow]) {
    out.push_str("\n## Included Files\n\n");
    if selected.is_empty() {
        out.push_str("- No files were bundled.\n");
        return;
    }
    for file in selected.iter().take(20) {
        let effective_tokens = file.effective_tokens.unwrap_or(file.tokens);
        out.push_str(&format!(
            "- `{}`: {}, policy `{}`, {} effective tokens",
            file.path,
            file.lang,
            policy_label(file.policy),
            effective_tokens
        ));
        if !file.rank_reason.is_empty() {
            out.push_str(&format!(", reason: {}", file.rank_reason));
        }
        out.push('\n');
    }
    if selected.len() > 20 {
        out.push_str(&format!(
            "- ... {} more bundled file(s); see `manifest.json` for the full list.\n",
            selected.len() - 20
        ));
    }
}

fn push_agent_stop_conditions_section(
    out: &mut String,
    links: &HandoffLinkInputs<'_>,
    summary: &LinkedEvidenceSummary,
) {
    out.push_str("\n## Agent Stop Conditions\n\n");
    out.push_str("- Stop after the requested repair or review slice is complete; do not broaden the lane by default.\n");
    if links.review_packet_check.is_some() {
        out.push_str(
            "- Stop if the linked review-packet verifier is missing, unreadable, or failing.\n",
        );
    }
    if let Some(affected) = &summary.affected
        && affected.unknown_files > 0
    {
        out.push_str("- Stop before claiming proof if affected routing still has unknown files.\n");
    }
    if links.proof_plan.is_some() {
        out.push_str("- Stop before claiming done until required proof commands are run or explicitly deferred.\n");
    }
    out.push_str("- Stop before promoting advisory proof, enabling Codecov defaults, adding AST defaults, or treating cockpit/handoff output as a merge verdict.\n");
}

fn push_agent_guardrails_section(out: &mut String) {
    out.push_str("\n## Agent Guardrails\n\n");
    out.push_str("- Treat missing, stale, degraded, skipped, or unavailable evidence as work to resolve, not as passing proof.\n");
    out.push_str("- Run reproduction commands from the linked review map before claiming a repair is proven.\n");
    out.push_str(
        "- Keep generated receipts with the work when they explain review or proof state.\n",
    );
    out.push_str("- Do not promote advisory proof, enable default Codecov upload, or turn this handoff into a merge verdict.\n");
}

fn review_links_json(
    review_packet_dir: Option<&Path>,
    review_packet_check: Option<&Path>,
) -> Value {
    let packet_artifacts = review_packet_dir
        .map(|dir| {
            [
                ("comment", "comment.md"),
                ("review_map_md", "review-map.md"),
                ("review_map_json", "review-map.json"),
                ("evidence", "evidence.json"),
                ("manifest", "manifest.json"),
                ("cockpit", "cockpit.json"),
            ]
            .into_iter()
            .map(|(name, relative)| path_link(name, &dir.join(relative)))
            .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    json!({
        "schema": "tokmd.handoff_review_links.v1",
        "review_packet_dir": review_packet_dir.map(path_string),
        "review_packet_check": review_packet_check.map(|path| path_link("review_packet_check", path)),
        "artifacts": packet_artifacts,
        "semantics": {
            "kind": "external_links",
            "copied": false,
            "integrity_source": "cargo xtask review-packet-check"
        }
    })
}

fn proof_links_json(affected: Option<&Path>, proof_plan: Option<&Path>) -> Value {
    let mut artifacts = Vec::new();
    if let Some(path) = affected {
        artifacts.push(path_link("affected", path));
    }
    if let Some(path) = proof_plan {
        artifacts.push(path_link("proof_plan", path));
    }

    json!({
        "schema": "tokmd.handoff_proof_links.v1",
        "artifacts": artifacts,
        "semantics": {
            "kind": "external_links",
            "copied": false,
            "integrity_source": "linked proof artifacts"
        }
    })
}

fn path_link(name: &str, path: &Path) -> Value {
    let bytes = path
        .metadata()
        .ok()
        .filter(|metadata| metadata.is_file())
        .map(|metadata| metadata.len());

    json!({
        "name": name,
        "path": path_string(path),
        "exists": path.exists(),
        "bytes": bytes,
    })
}

fn path_string(path: &Path) -> String {
    path.display().to_string().replace('\\', "/")
}

fn policy_label(policy: InclusionPolicy) -> &'static str {
    match policy {
        InclusionPolicy::Full => "full",
        InclusionPolicy::HeadTail => "head_tail",
        InclusionPolicy::Summary => "summary",
        InclusionPolicy::Skip => "skip",
    }
}

fn write_map_jsonl(path: &Path, export: &ExportData) -> Result<u64> {
    let file =
        File::create(path).with_context(|| format!("Failed to create {}", path.display()))?;
    let mut writer = std::io::BufWriter::new(file);
    let mut bytes: u64 = 0;

    for row in export.rows.iter().filter(|r| r.kind == FileKind::Parent) {
        let json = serde_json::to_string(row)?;
        writeln!(writer, "{}", json)?;
        bytes += json.len() as u64 + 1;
    }

    writer.flush()?;
    Ok(bytes)
}

fn write_code_bundle(path: &Path, selected: &[ContextFileRow], compress: bool) -> Result<u64> {
    let file =
        File::create(path).with_context(|| format!("Failed to create {}", path.display()))?;
    let mut writer = std::io::BufWriter::new(file);
    let mut bytes: u64 = 0;

    for ctx_file in selected {
        let file_path = PathBuf::from(&ctx_file.path);
        if !file_path.exists() {
            continue;
        }

        match ctx_file.policy {
            InclusionPolicy::Full => {
                let header = format!("// === {} ===\n", ctx_file.path);
                writer.write_all(header.as_bytes())?;
                bytes += header.len() as u64;

                if compress {
                    let file = File::open(&file_path)
                        .with_context(|| format!("Failed to open file: {}", file_path.display()))?;
                    let reader = BufReader::new(file);
                    for line in reader.lines() {
                        let line = line.with_context(|| {
                            format!("Failed to read file: {}", file_path.display())
                        })?;
                        if !line.trim().is_empty() {
                            writeln!(writer, "{}", line)?;
                            bytes += line.len() as u64 + 1;
                        }
                    }
                    writeln!(writer)?;
                    bytes += 1;
                } else {
                    let content = fs::read_to_string(&file_path)
                        .with_context(|| format!("Failed to read file: {}", file_path.display()))?;
                    writer.write_all(content.as_bytes())?;
                    bytes += content.len() as u64;
                    if !content.ends_with('\n') {
                        writeln!(writer)?;
                        bytes += 1;
                    }
                    writeln!(writer)?;
                    bytes += 1;
                }
            }
            InclusionPolicy::HeadTail => {
                let header = format!("// === {} ===\n", ctx_file.path);
                writer.write_all(header.as_bytes())?;
                bytes += header.len() as u64;

                let mut buf = Vec::new();
                crate::context_pack::write_head_tail(&mut buf, &file_path, ctx_file, compress)?;
                writer.write_all(&buf)?;
                bytes += buf.len() as u64;

                writeln!(writer)?;
                bytes += 1;
            }
            InclusionPolicy::Summary | InclusionPolicy::Skip => {
                let header = format!(
                    "// === {} [skipped: {}] ===\n\n",
                    ctx_file.path,
                    ctx_file.policy_reason.as_deref().unwrap_or("policy")
                );
                writer.write_all(header.as_bytes())?;
                bytes += header.len() as u64;
            }
        }
    }

    writer.flush()?;
    Ok(bytes)
}

fn hash_bytes(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

fn hash_file(path: &Path) -> Result<String> {
    let mut file =
        File::open(path).with_context(|| format!("Failed to open {}", path.display()))?;
    let mut hasher = Hasher::new();
    let mut buf = [0u8; 8 * 1024];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hasher.finalize().to_hex().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokmd_types::{ChildIncludeMode, FileRow};

    #[test]
    fn map_jsonl_writes_parent_rows_only() {
        let temp = tempfile::tempdir().expect("tempdir");
        let path = temp.path().join("map.jsonl");
        let export = ExportData {
            rows: vec![
                FileRow {
                    path: "src/lib.rs".to_string(),
                    module: "src".to_string(),
                    lang: "Rust".to_string(),
                    kind: FileKind::Parent,
                    code: 1,
                    comments: 0,
                    blanks: 0,
                    lines: 1,
                    bytes: 10,
                    tokens: 3,
                },
                FileRow {
                    path: "src/lib.rs:Markdown".to_string(),
                    module: "src".to_string(),
                    lang: "Markdown".to_string(),
                    kind: FileKind::Child,
                    code: 99,
                    comments: 0,
                    blanks: 0,
                    lines: 99,
                    bytes: 99,
                    tokens: 99,
                },
            ],
            module_roots: vec![],
            module_depth: 2,
            children: ChildIncludeMode::ParentsOnly,
        };

        let bytes = write_map_jsonl(&path, &export).expect("write map");
        let contents = fs::read_to_string(path).expect("read map");

        assert!(bytes > 0);
        assert_eq!(contents.lines().count(), 1);
        assert!(contents.contains("src/lib.rs"));
        assert!(!contents.contains("Markdown"));
    }

    #[test]
    fn hash_file_matches_hash_bytes() {
        let temp = tempfile::tempdir().expect("tempdir");
        let path = temp.path().join("artifact.txt");
        fs::write(&path, b"receipt-grade").expect("write fixture");

        let from_file = hash_file(&path).expect("hash file");
        let from_bytes = hash_bytes(b"receipt-grade");

        assert_eq!(from_file, from_bytes);
    }
}
