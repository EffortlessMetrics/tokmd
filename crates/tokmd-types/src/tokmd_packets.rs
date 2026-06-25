//! Cross-tool packet bundle input types for `tokmd render`.
//!
//! These types model the `tokmd-packets.json` contract consumed when rendering
//! audience-specific packet presets from unsafe-review manual-candidate bundles.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Stable schema identifier for packet bundle manifests.
pub const TOKMD_PACKETS_SCHEMA: &str = "tokmd.packets/v1";

/// Bun UB packet presets documented in `docs/specs/tokmd-packets-render.md`.
pub const BUN_UB_PACKET_PRESETS: &[&str] = &[
    "bun-ub-handoff",
    "bun-ub-pr-body",
    "bun-ub-ledger-note",
    "bun-ub-review-map",
    "bun-ub-next-pick",
];

/// Top-level packet bundle manifest (`tokmd-packets.json`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokmdPacketsManifest {
    pub schema: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub producer: Option<TokmdPacketsProducer>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inputs_present: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inputs_absent: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub non_claims: Vec<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub preset_inputs: BTreeMap<String, PacketPresetInput>,
}

/// Producer metadata recorded by the exporting tool.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokmdPacketsProducer {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokmd_run: Option<bool>,
}

/// Ready-to-format sections for one audience preset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PacketPresetInput {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub sections: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub limitations: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub missing_sections: Vec<String>,
}

impl TokmdPacketsManifest {
    /// Returns true when the manifest uses the supported schema id.
    pub fn schema_matches(&self) -> bool {
        self.schema == TOKMD_PACKETS_SCHEMA
    }

    /// Returns true when `preset` is a known Bun UB packet preset name.
    pub fn preset_is_known(preset: &str) -> bool {
        BUN_UB_PACKET_PRESETS.contains(&preset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokmd_packets_schema_constant() {
        assert_eq!(TOKMD_PACKETS_SCHEMA, "tokmd.packets/v1");
    }

    #[test]
    fn bun_ub_presets_are_stable() {
        assert_eq!(BUN_UB_PACKET_PRESETS.len(), 5);
        assert!(BUN_UB_PACKET_PRESETS.contains(&"bun-ub-handoff"));
    }

    #[test]
    fn manifest_roundtrip_preserves_preset_inputs() {
        let mut sections = BTreeMap::new();
        sections.insert("candidate_identity".into(), "seed-42".into());
        let manifest = TokmdPacketsManifest {
            schema: TOKMD_PACKETS_SCHEMA.into(),
            producer: Some(TokmdPacketsProducer {
                name: "unsafe-review".into(),
                version: Some("0.1.0".into()),
                tokmd_run: Some(false),
            }),
            inputs_present: vec!["manual-candidates.json".into()],
            inputs_absent: vec!["comment-plan.json".into()],
            non_claims: vec!["Does not prove UB.".into()],
            preset_inputs: BTreeMap::from([(
                "bun-ub-handoff".into(),
                PacketPresetInput {
                    sections,
                    limitations: vec!["witness-plan.md absent".into()],
                    missing_sections: vec!["test or witness target".into()],
                },
            )]),
        };
        let json = serde_json::to_string(&manifest).unwrap();
        let back: TokmdPacketsManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(back, manifest);
        assert!(back.schema_matches());
    }
}
