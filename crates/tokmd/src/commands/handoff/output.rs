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

#[derive(Default)]
struct LinkedEvidenceSummary {
    review_map: Option<ReviewMapSummary>,
    review_packet_check: Option<ReviewPacketCheckSummary>,
    affected: Option<AffectedSummary>,
    proof_plan: Option<ProofPlanSummary>,
}

struct ReviewMapSummary {
    item_count: usize,
    first_items: Vec<ReviewMapItemSummary>,
    available: Option<u64>,
    missing: Option<u64>,
    degraded: Option<u64>,
    stale: Option<u64>,
    skipped: Option<u64>,
    unavailable: Option<u64>,
}

struct ReviewMapItemSummary {
    path: String,
    reason: Option<String>,
}

struct ReviewPacketCheckSummary {
    ok: Option<bool>,
    artifact_count: Option<u64>,
    hashes_verified: Option<u64>,
}

struct AffectedSummary {
    changed_files: usize,
    scopes: usize,
    unknown_files: usize,
    scope_names: Vec<String>,
}

struct ProofPlanSummary {
    commands: usize,
    required: usize,
    advisory: usize,
    first_commands: Vec<String>,
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
    let linked_evidence = summarize_linked_evidence(order.links);
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
    out.push_str("# Agent Work Order\n\n");
    out.push_str("This handoff is a deterministic source/context bundle for coding-agent work.\n");
    out.push_str("Treat linked review and proof receipts as external evidence handles; this file does not verify them.\n\n");

    out.push_str("## Start Here\n\n");
    let mut steps = vec![
        "Read `manifest.json` for the authoritative artifact index, token budget, included files, and exclusions.",
        "Read `work-order.md` for the agent task map and guardrails.",
        "Read `code.txt` for the bounded source bundle.",
        "Use `map.jsonl` for full file inventory and path lookup.",
        "Use `intelligence.json` for repository shape, hotspots, complexity, and derived signals.",
    ];
    if order.links.review_packet_dir.is_some() || order.links.review_packet_check.is_some() {
        steps.push(
            "Use `review-links.json` for cockpit review packet and verifier receipt pointers.",
        );
    }
    if order.links.affected.is_some() || order.links.proof_plan.is_some() {
        steps.push("Use `proof-links.json` for affected-proof and proof-plan pointers.");
    }
    for (index, step) in steps.iter().enumerate() {
        out.push_str(&format!("{}. {}\n", index + 1, step));
    }

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

    out.push_str("\n## Linked Evidence\n\n");
    if let Some(path) = order.links.review_packet_dir {
        out.push_str(&format!(
            "- Review packet directory: `{}`\n",
            path_string(path)
        ));
    } else {
        out.push_str("- Review packet directory: not linked\n");
    }
    if let Some(path) = order.links.review_packet_check {
        out.push_str(&format!(
            "- Review packet verifier receipt: `{}`\n",
            path_string(path)
        ));
    } else {
        out.push_str("- Review packet verifier receipt: not linked\n");
    }
    if let Some(path) = order.links.affected {
        out.push_str(&format!(
            "- Affected proof report: `{}`\n",
            path_string(path)
        ));
    } else {
        out.push_str("- Affected proof report: not linked\n");
    }
    if let Some(path) = order.links.proof_plan {
        out.push_str(&format!("- Proof plan report: `{}`\n", path_string(path)));
    } else {
        out.push_str("- Proof plan report: not linked\n");
    }

    render_linked_evidence_summary(&mut out, order.links, linked_evidence);

    out.push_str("\n## Included Files\n\n");
    if order.selected.is_empty() {
        out.push_str("- No files were bundled.\n");
    } else {
        for file in order.selected.iter().take(20) {
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
        if order.selected.len() > 20 {
            out.push_str(&format!(
                "- ... {} more bundled file(s); see `manifest.json` for the full list.\n",
                order.selected.len() - 20
            ));
        }
    }

    out.push_str("\n## Agent Guardrails\n\n");
    out.push_str("- Treat missing, stale, degraded, skipped, or unavailable evidence as work to resolve, not as passing proof.\n");
    out.push_str("- Run reproduction commands from the linked review map before claiming a repair is proven.\n");
    out.push_str(
        "- Keep generated receipts with the work when they explain review or proof state.\n",
    );
    out.push_str("- Do not promote advisory proof, enable default Codecov upload, or turn this handoff into a merge verdict.\n");

    out
}

fn render_linked_evidence_summary(
    out: &mut String,
    links: &HandoffLinkInputs<'_>,
    summary: &LinkedEvidenceSummary,
) {
    if !has_any_link(links) {
        return;
    }

    out.push_str("\n## Linked Evidence Summary\n\n");
    out.push_str("These summaries are best-effort hints from linked receipts. They do not replace the linked verifier or proof artifacts.\n\n");

    if let Some(check) = &summary.review_packet_check {
        out.push_str("- Review packet verifier:");
        if let Some(ok) = check.ok {
            out.push_str(&format!(" ok={ok}"));
        } else {
            out.push_str(" ok=unknown");
        }
        if let Some(artifact_count) = check.artifact_count {
            out.push_str(&format!(", artifacts={artifact_count}"));
        }
        if let Some(hashes_verified) = check.hashes_verified {
            out.push_str(&format!(", hashes_verified={hashes_verified}"));
        }
        out.push('\n');
    } else if links.review_packet_check.is_some() {
        out.push_str("- Review packet verifier: linked but not readable\n");
    }

    if let Some(review_map) = &summary.review_map {
        out.push_str(&format!("- Review map: {} item(s)", review_map.item_count));
        if review_map.available.is_some()
            || review_map.missing.is_some()
            || review_map.degraded.is_some()
            || review_map.stale.is_some()
            || review_map.skipped.is_some()
            || review_map.unavailable.is_some()
        {
            out.push_str(" (");
            push_count(out, "available", review_map.available);
            push_count(out, "missing", review_map.missing);
            push_count(out, "degraded", review_map.degraded);
            push_count(out, "stale", review_map.stale);
            push_count(out, "skipped", review_map.skipped);
            push_count(out, "unavailable", review_map.unavailable);
            trim_trailing_separator(out);
            out.push(')');
        }
        out.push('\n');
        if !review_map.first_items.is_empty() {
            out.push_str("  - Review first:\n");
            for item in &review_map.first_items {
                out.push_str(&format!("    - `{}`", item.path));
                if let Some(reason) = &item.reason {
                    out.push_str(&format!(": {reason}"));
                }
                out.push('\n');
            }
        }
    } else if links.review_packet_dir.is_some() {
        out.push_str("- Review map: linked but not readable\n");
    }

    if let Some(affected) = &summary.affected {
        out.push_str(&format!(
            "- Affected proof: {} changed file(s), {} scope(s), {} unknown file(s)\n",
            affected.changed_files, affected.scopes, affected.unknown_files
        ));
        if !affected.scope_names.is_empty() {
            out.push_str("  - Scopes: ");
            out.push_str(&affected.scope_names.join(", "));
            out.push('\n');
        }
    } else if links.affected.is_some() {
        out.push_str("- Affected proof: linked but not readable\n");
    }

    if let Some(proof_plan) = &summary.proof_plan {
        out.push_str(&format!(
            "- Proof plan: {} command(s), {} required, {} advisory\n",
            proof_plan.commands, proof_plan.required, proof_plan.advisory
        ));
        if !proof_plan.first_commands.is_empty() {
            out.push_str("  - First commands:\n");
            for command in &proof_plan.first_commands {
                out.push_str(&format!("    - `{command}`\n"));
            }
            if proof_plan.commands > proof_plan.first_commands.len() {
                out.push_str(&format!(
                    "    - ... {} more command(s); open the proof plan for the full list.\n",
                    proof_plan.commands - proof_plan.first_commands.len()
                ));
            }
        }
        out.push_str("  - A proof plan is planned evidence, not execution proof.\n");
    } else if links.proof_plan.is_some() {
        out.push_str("- Proof plan: linked but not readable\n");
    }
}

fn summarize_linked_evidence(links: &HandoffLinkInputs<'_>) -> LinkedEvidenceSummary {
    LinkedEvidenceSummary {
        review_map: links
            .review_packet_dir
            .and_then(|dir| read_json_value(&dir.join("review-map.json")))
            .and_then(|value| summarize_review_map(&value)),
        review_packet_check: links
            .review_packet_check
            .and_then(read_json_value)
            .map(|value| summarize_review_packet_check(&value)),
        affected: links
            .affected
            .and_then(read_json_value)
            .map(|value| summarize_affected(&value)),
        proof_plan: links
            .proof_plan
            .and_then(read_json_value)
            .map(|value| summarize_proof_plan(&value)),
    }
}

fn read_json_value(path: &Path) -> Option<Value> {
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

fn summarize_review_map(value: &Value) -> Option<ReviewMapSummary> {
    let items = value.get("items")?.as_array()?;
    let item_count = value
        .get("item_count")
        .and_then(Value::as_u64)
        .map(|count| count as usize)
        .unwrap_or(items.len());
    let first_items = items
        .iter()
        .take(5)
        .filter_map(|item| {
            let path = item.get("path")?.as_str()?.to_string();
            let reason = item
                .get("reason")
                .and_then(Value::as_str)
                .map(str::to_string);
            Some(ReviewMapItemSummary { path, reason })
        })
        .collect();
    let evidence_summary = value.get("evidence").and_then(|e| e.get("summary"));

    Some(ReviewMapSummary {
        item_count,
        first_items,
        available: count_field(evidence_summary, "available"),
        missing: count_field(evidence_summary, "missing"),
        degraded: count_field(evidence_summary, "degraded"),
        stale: count_field(evidence_summary, "stale"),
        skipped: count_field(evidence_summary, "skipped"),
        unavailable: count_field(evidence_summary, "unavailable"),
    })
}

fn summarize_review_packet_check(value: &Value) -> ReviewPacketCheckSummary {
    ReviewPacketCheckSummary {
        ok: value.get("ok").and_then(Value::as_bool),
        artifact_count: value.get("artifact_count").and_then(Value::as_u64),
        hashes_verified: value.get("hashes_verified").and_then(Value::as_u64),
    }
}

fn summarize_affected(value: &Value) -> AffectedSummary {
    let changed_files = array_len(value.get("changed_files"));
    let scopes_array = value.get("scopes").and_then(Value::as_array);
    let scope_names = scopes_array
        .into_iter()
        .flat_map(|scopes| scopes.iter())
        .filter_map(|scope| scope.get("name").and_then(Value::as_str))
        .take(8)
        .map(str::to_string)
        .collect::<Vec<_>>();

    AffectedSummary {
        changed_files,
        scopes: array_len(value.get("scopes")),
        unknown_files: array_len(value.get("unknown_files")),
        scope_names,
    }
}

fn summarize_proof_plan(value: &Value) -> ProofPlanSummary {
    let Some(commands) = value.get("commands").and_then(Value::as_array) else {
        return ProofPlanSummary {
            commands: 0,
            required: 0,
            advisory: 0,
            first_commands: Vec::new(),
        };
    };
    let required = commands
        .iter()
        .filter(|command| command.get("required").and_then(Value::as_bool) == Some(true))
        .count();
    let advisory = commands.len().saturating_sub(required);
    let first_commands = commands
        .iter()
        .filter_map(|command| command.get("command").and_then(Value::as_str))
        .take(5)
        .map(str::to_string)
        .collect();

    ProofPlanSummary {
        commands: commands.len(),
        required,
        advisory,
        first_commands,
    }
}

fn has_any_link(links: &HandoffLinkInputs<'_>) -> bool {
    links.review_packet_dir.is_some()
        || links.review_packet_check.is_some()
        || links.affected.is_some()
        || links.proof_plan.is_some()
}

fn count_field(value: Option<&Value>, field: &str) -> Option<u64> {
    value
        .and_then(|value| value.get(field))
        .and_then(Value::as_u64)
}

fn array_len(value: Option<&Value>) -> usize {
    value.and_then(Value::as_array).map_or(0, Vec::len)
}

fn push_count(out: &mut String, label: &str, count: Option<u64>) {
    if let Some(count) = count {
        out.push_str(&format!("{label}={count}, "));
    }
}

fn trim_trailing_separator(out: &mut String) {
    if out.ends_with(", ") {
        out.truncate(out.len() - 2);
    }
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
