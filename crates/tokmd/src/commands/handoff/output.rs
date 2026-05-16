//! Handoff bundle output writers.

use std::path::Path;

use tokmd_types::{ArtifactEntry, ContextFileRow};

mod artifact;
mod code_bundle;
mod hash;
mod linked_evidence;
mod links;
mod manifest;
mod payloads;
mod work_order;

#[cfg(test)]
use code_bundle::write_map_jsonl;
pub(super) use links::write_link_artifacts;
pub(super) use manifest::write_manifest_json;
pub(super) use payloads::write_payloads;
pub(super) use work_order::write_work_order;

#[cfg(test)]
use hash::{hash_bytes, hash_file};

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

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
