//! Top-level payload artifact assembly.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use tokmd_types::{ArtifactEntry, ArtifactHash, ContextFileRow, ExportData, HandoffIntelligence};

use super::HandoffPayloads;
use super::code_bundle::{write_code_bundle, write_map_jsonl};
use super::hash::{hash_bytes, hash_file};

pub(crate) fn write_payloads(
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
        hashed_artifact(
            "map",
            "map.jsonl",
            "Complete file inventory",
            map_bytes,
            map_hash,
        ),
        hashed_artifact(
            "intelligence",
            "intelligence.json",
            "Tree, hotspots, complexity, and derived metrics",
            intelligence_bytes,
            intelligence_hash,
        ),
        hashed_artifact(
            "code",
            "code.txt",
            "Token-budgeted code bundle",
            code_bytes,
            code_hash,
        ),
    ];

    Ok(HandoffPayloads {
        map_bytes,
        intelligence_bytes,
        code_bytes,
        artifacts,
    })
}

fn hashed_artifact(
    name: &str,
    path: &str,
    description: &str,
    bytes: u64,
    hash: String,
) -> ArtifactEntry {
    ArtifactEntry {
        name: name.to_string(),
        path: path.to_string(),
        description: description.to_string(),
        bytes,
        hash: Some(ArtifactHash {
            algo: "blake3".to_string(),
            hash,
        }),
    }
}
