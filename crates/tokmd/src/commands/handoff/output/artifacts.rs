//! Shared artifact writing and hashing helpers for handoff output.

use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

use anyhow::{Context, Result};
use blake3::Hasher;
use serde_json::Value;
use tokmd_types::{ArtifactEntry, ArtifactHash};

pub(super) fn write_json_artifact(
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

pub(super) fn write_text_artifact(
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

pub(super) fn hash_bytes(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

pub(super) fn hash_file(path: &Path) -> Result<String> {
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
