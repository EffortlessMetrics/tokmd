//! Handoff bundle output writers.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use tokmd_types::{
    ArtifactEntry, ArtifactHash, ContextFileRow, ExportData, HandoffIntelligence, HandoffManifest,
};

mod artifacts;
mod bundle;
mod links;
mod work_order;

use artifacts::{hash_bytes, hash_file};
use bundle::{write_code_bundle, write_map_jsonl};

pub(super) use links::HandoffLinkInputs;
pub(super) use work_order::HandoffWorkOrderInputs;

pub(super) struct HandoffPayloads {
    pub(super) map_bytes: u64,
    pub(super) intelligence_bytes: u64,
    pub(super) code_bytes: u64,
    pub(super) artifacts: Vec<ArtifactEntry>,
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
    links::write_link_artifacts(out_dir, links)
}

pub(super) fn write_work_order(
    out_dir: &Path,
    order: &HandoffWorkOrderInputs<'_>,
) -> Result<ArtifactEntry> {
    work_order::write_work_order(out_dir, order)
}

pub(super) fn write_manifest_json(out_dir: &Path, manifest: &HandoffManifest) -> Result<usize> {
    let manifest_path = out_dir.join("manifest.json");
    let manifest_json = serde_json::to_string_pretty(manifest)?;
    fs::write(&manifest_path, &manifest_json)
        .with_context(|| format!("Failed to write {}", manifest_path.display()))?;
    Ok(manifest_json.len())
}
