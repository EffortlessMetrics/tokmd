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
