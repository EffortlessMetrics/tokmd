//! Documentation-control evidence imported into cockpit review packets.

use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub(crate) const DOC_ARTIFACTS_CHECK_SCHEMA: &str = "tokmd.doc_artifacts_check.v1";
pub(crate) const DOC_ARTIFACTS_PACKET_PATH: &str = "docs/doc-artifacts-check.json";

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct DocArtifactsCheckReceipt {
    pub schema: String,
    pub ok: bool,
    pub checked: DocArtifactsCheckedCounts,
    #[serde(default)]
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct DocArtifactsCheckedCounts {
    pub required_docs: usize,
    pub family_files: usize,
    pub active_goals: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocArtifactsEvidenceInput {
    pub source_path: PathBuf,
    pub receipt: DocArtifactsCheckReceipt,
}

impl DocArtifactsEvidenceInput {
    pub(crate) fn availability(&self) -> &'static str {
        if self.receipt.ok {
            "available"
        } else {
            "degraded"
        }
    }
}

/// Parse a documentation artifact checker receipt with its source path.
pub fn parse_doc_artifacts_evidence_input(
    raw: &str,
    source_path: impl Into<PathBuf>,
) -> Result<DocArtifactsEvidenceInput> {
    let value: Value = serde_json::from_str(raw).context("parse doc artifacts evidence JSON")?;
    let schema = value
        .get("schema")
        .and_then(Value::as_str)
        .context("doc artifacts evidence artifact missing string schema")?;
    if schema != DOC_ARTIFACTS_CHECK_SCHEMA {
        bail!("unsupported doc artifacts evidence schema `{schema}`");
    }

    let receipt: DocArtifactsCheckReceipt =
        serde_json::from_value(value).context("parse doc artifacts check evidence")?;
    Ok(DocArtifactsEvidenceInput {
        source_path: source_path.into(),
        receipt,
    })
}

pub(crate) fn source_of_truth_path(path: &str) -> bool {
    path.starts_with(".tokmd-spec/")
        || path == "docs/source-of-truth.md"
        || path == "docs/review-packet.md"
        || path == "docs/cockpit-proof-evidence.md"
        || path == "docs/agent-workflows/source-of-truth.md"
        || path == "docs/ci/swarm-routing.md"
        || path == "docs/contributing/spec-rails.md"
        || path == "docs/spec-style.md"
        || path == "policy/doc-artifacts.toml"
        || path.starts_with("docs/proposals/")
        || path.starts_with("docs/specs/")
        || path.starts_with("docs/adr/")
        || path.starts_with("docs/plans/")
        || path.starts_with("docs/templates/")
        || path.starts_with(".jules/goals/")
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_RECEIPT: &str = r#"{
  "schema": "tokmd.doc_artifacts_check.v1",
  "ok": true,
  "checked": {
    "required_docs": 1,
    "family_files": 11,
    "active_goals": 1
  },
  "errors": []
}"#;

    #[test]
    fn parses_doc_artifacts_check_receipt() {
        let input = parse_doc_artifacts_evidence_input(VALID_RECEIPT, "target/docs/check.json")
            .expect("valid doc artifacts receipt");

        assert_eq!(input.receipt.schema, DOC_ARTIFACTS_CHECK_SCHEMA);
        assert_eq!(input.receipt.checked.required_docs, 1);
        assert_eq!(input.receipt.checked.family_files, 11);
        assert_eq!(input.receipt.checked.active_goals, 1);
        assert_eq!(input.availability(), "available");
    }

    #[test]
    fn rejects_unknown_doc_artifacts_schema() {
        let err = parse_doc_artifacts_evidence_input(
            r#"{ "schema": "tokmd.unknown.v1" }"#,
            "target/docs/check.json",
        )
        .expect_err("unknown schema should fail");

        assert!(
            err.to_string()
                .contains("unsupported doc artifacts evidence schema `tokmd.unknown.v1`")
        );
    }

    #[test]
    fn recognizes_source_of_truth_paths() {
        assert!(source_of_truth_path(".tokmd-spec/README.md"));
        assert!(source_of_truth_path(".tokmd-spec/index.toml"));
        assert!(source_of_truth_path("docs/source-of-truth.md"));
        assert!(source_of_truth_path("docs/review-packet.md"));
        assert!(source_of_truth_path("docs/cockpit-proof-evidence.md"));
        assert!(source_of_truth_path(
            "docs/agent-workflows/source-of-truth.md"
        ));
        assert!(source_of_truth_path("docs/ci/swarm-routing.md"));
        assert!(source_of_truth_path("docs/contributing/spec-rails.md"));
        assert!(source_of_truth_path("docs/spec-style.md"));
        assert!(source_of_truth_path("docs/specs/doc-artifacts.md"));
        assert!(source_of_truth_path(".jules/goals/active.toml"));
        assert!(source_of_truth_path("policy/doc-artifacts.toml"));
        assert!(!source_of_truth_path("crates/tokmd/src/main.rs"));
    }
}
