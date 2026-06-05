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
}

/// Evidence packet status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidencePacketStatus {
    Complete,
    Partial,
    Failed,
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
            },
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
}
