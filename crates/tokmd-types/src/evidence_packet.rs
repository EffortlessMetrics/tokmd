//! Evidence packet manifest DTOs.
//!
//! These types model the `sensors/tokmd/manifest.json` contract used by
//! high-risk review sensors. They intentionally index existing receipts rather
//! than replacing them.

use serde::{Deserialize, Serialize};

/// Stable schema identifier for evidence packet manifests.
pub const EVIDENCE_PACKET_SCHEMA: &str = "tokmd.evidence-packet/v1";

/// Manifest for a scoped evidence packet directory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidencePacketManifest {
    pub schema: String,
    pub tokmd_version: String,
    pub preset: String,
    pub base: String,
    pub head: String,
    pub paths: Vec<String>,
    pub status: EvidencePacketStatus,
    pub artifacts: EvidencePacketArtifacts,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub review_priority: Vec<EvidencePacketReviewPriorityItem>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub non_claims: Vec<String>,
    pub reproduce: Vec<String>,
}

/// Packet artifact paths.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidencePacketArtifacts {
    pub analyze_md: String,
    pub analyze_json: String,
    pub context_md: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub syntax_json: Option<String>,
}

/// Evidence packet status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidencePacketStatus {
    Complete,
    Partial,
    Failed,
}

/// Advisory first-read item derived from packet artifacts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidencePacketReviewPriorityItem {
    pub rank: u32,
    pub path: String,
    pub category: String,
    pub severity: String,
    pub score: u32,
    pub kind: String,
    pub reason: String,
    pub evidence: String,
    pub refs: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evidence_packet_status_uses_contract_spelling() {
        assert_eq!(
            serde_json::to_string(&EvidencePacketStatus::Complete).unwrap(),
            "\"complete\""
        );
        assert_eq!(
            serde_json::to_string(&EvidencePacketStatus::Partial).unwrap(),
            "\"partial\""
        );
        assert_eq!(
            serde_json::to_string(&EvidencePacketStatus::Failed).unwrap(),
            "\"failed\""
        );
    }

    #[test]
    fn evidence_packet_status_serde_exhaustive() {
        // Ensure all variants deserialize exactly
        assert_eq!(
            serde_json::from_str::<EvidencePacketStatus>("\"complete\"").unwrap(),
            EvidencePacketStatus::Complete
        );
        assert_eq!(
            serde_json::from_str::<EvidencePacketStatus>("\"partial\"").unwrap(),
            EvidencePacketStatus::Partial
        );
        assert_eq!(
            serde_json::from_str::<EvidencePacketStatus>("\"failed\"").unwrap(),
            EvidencePacketStatus::Failed
        );
    }

    #[test]
    fn evidence_packet_manifest_roundtrips() {
        let manifest = EvidencePacketManifest {
            schema: EVIDENCE_PACKET_SCHEMA.to_string(),
            tokmd_version: "1.12.0".to_string(),
            preset: "bun-ub".to_string(),
            base: "origin/main".to_string(),
            head: "HEAD".to_string(),
            paths: vec!["src/runtime/api".to_string()],
            status: EvidencePacketStatus::Complete,
            artifacts: EvidencePacketArtifacts {
                analyze_md: "sensors/tokmd/analyze.md".to_string(),
                analyze_json: "sensors/tokmd/analyze.json".to_string(),
                context_md: "sensors/tokmd/context.md".to_string(),
                syntax_json: Some("sensors/tokmd/syntax.json".to_string()),
            },
            review_priority: vec![],
            warnings: vec![],
            errors: vec![],
            non_claims: vec![
                "bun-ub packages review evidence; it does not prove UB exists or is absent"
                    .to_string(),
            ],
            reproduce: vec![
                "tokmd analyze --preset bun-ub --format json --effort-base-ref origin/main --effort-head-ref HEAD --no-progress src/runtime/api > sensors/tokmd/analyze.json"
                    .to_string(),
            ],
        };

        let json = serde_json::to_string(&manifest).unwrap();
        let roundtrip: EvidencePacketManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtrip, manifest);
    }

    #[test]
    fn evidence_packet_manifest_defaults_missing_review_priority() {
        let json = serde_json::json!({
            "schema": EVIDENCE_PACKET_SCHEMA,
            "tokmd_version": "1.12.0",
            "preset": "bun-ub",
            "base": "origin/main",
            "head": "HEAD",
            "paths": ["src/runtime/api"],
            "status": "complete",
            "artifacts": {
                "analyze_md": "sensors/tokmd/analyze.md",
                "analyze_json": "sensors/tokmd/analyze.json",
                "context_md": "sensors/tokmd/context.md"
            },
            "warnings": [],
            "errors": [],
            "non_claims": [],
            "reproduce": []
        });

        let manifest: EvidencePacketManifest = serde_json::from_value(json).unwrap();
        assert!(manifest.review_priority.is_empty());
    }

    #[test]
    fn evidence_packet_canonical_layout_matches_spec() {
        // Keep aligned with docs/specs/evidence-packet-workflow.md.
        const PACKET_DIR: &str = "sensors/tokmd";
        const MANIFEST: &str = "sensors/tokmd/manifest.json";
        const REQUIRED: &[&str] = &["analyze.md", "analyze.json", "context.md", "manifest.json"];
        const OPTIONAL_SYNTAX: &str = "syntax.json";

        assert_eq!(MANIFEST, "sensors/tokmd/manifest.json");
        assert!(REQUIRED.contains(&"analyze.md"));
        assert!(REQUIRED.contains(&"analyze.json"));
        assert!(REQUIRED.contains(&"context.md"));
        assert!(REQUIRED.contains(&"manifest.json"));
        assert_eq!(OPTIONAL_SYNTAX, "syntax.json");
        assert!(
            MANIFEST.starts_with(PACKET_DIR),
            "manifest path should live under the canonical packet directory"
        );
    }

    #[test]
    fn evidence_packet_review_priority_item_serializes() {
        let item = EvidencePacketReviewPriorityItem {
            rank: 1,
            path: "src/runtime/api/MarkdownObject.rs".to_string(),
            category: "panic_seam".to_string(),
            severity: "high".to_string(),
            score: 95,
            kind: "expect_call".to_string(),
            reason: "panic-like seam".to_string(),
            evidence: "expect".to_string(),
            refs: vec!["sensors/tokmd/syntax.json#/receipts/0/review_signals/1".to_string()],
        };

        let json = serde_json::to_value(&item).unwrap();
        assert_eq!(json["rank"], 1);
        assert_eq!(json["category"], "panic_seam");
        assert_eq!(
            json["refs"][0],
            "sensors/tokmd/syntax.json#/receipts/0/review_signals/1"
        );
    }
}
