//! Sibling bundle file types for `tokmd render --from-packets`.
//!
//! These model producer artifacts such as `manual-candidates.json` and
//! `cards.json` that may supplement manifest `preset_inputs` when partial.

use serde::{Deserialize, Serialize};

/// Sibling artifact filename for manual candidate indexes.
pub const MANUAL_CANDIDATES_FILE: &str = "manual-candidates.json";

/// Sibling artifact filename for ReviewCard analyzer output.
pub const CARDS_FILE: &str = "cards.json";

/// Schema id recorded by unsafe-review `manual-candidates.json` exports.
pub const MANUAL_CANDIDATES_SCHEMA: &str = "manual-candidates/v1";

/// Parsed sibling inputs available to the packet renderer.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PacketSiblingInputs {
    pub manual_candidates: Option<ManualCandidatesFile>,
    pub cards: Option<CardsFile>,
    pub load_notes: Vec<String>,
}

/// Bundle passed from CLI load through formatting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketRenderBundle {
    pub manifest: super::TokmdPacketsManifest,
    pub siblings: PacketSiblingInputs,
}

/// `manual-candidates.json` index (minimal consumer contract).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManualCandidatesFile {
    pub schema_version: String,
    #[serde(default)]
    pub candidates: Vec<ManualCandidateRecord>,
}

/// One manual candidate row from `manual-candidates.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ManualCandidateRecord {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub invariant: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub safe_caller: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unsafe_operation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation_family: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location_text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proof_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fix_boundary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pr_aperture: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub test_targets: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub do_not_touch: Vec<String>,
}

/// `cards.json` ReviewCard snapshot (minimal consumer contract).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CardsFile {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema_version: Option<String>,
    #[serde(default, alias = "review_cards")]
    pub cards: Vec<ReviewCardRecord>,
}

/// One ReviewCard row from `cards.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ReviewCardRecord {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location_text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unsafe_operation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation_family: Option<String>,
}

impl ManualCandidatesFile {
    /// Returns true when the file uses the supported schema id.
    pub fn schema_matches(&self) -> bool {
        self.schema_version == MANUAL_CANDIDATES_SCHEMA
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manual_candidates_roundtrip() {
        let file = ManualCandidatesFile {
            schema_version: MANUAL_CANDIDATES_SCHEMA.into(),
            candidates: vec![ManualCandidateRecord {
                id: Some("seed-42".into()),
                title: Some("UTF-8 boundary".into()),
                invariant: Some("buffer bounds".into()),
                ..ManualCandidateRecord::default()
            }],
        };
        let json = serde_json::to_string(&file).unwrap();
        let back: ManualCandidatesFile = serde_json::from_str(&json).unwrap();
        assert_eq!(back, file);
        assert!(back.schema_matches());
    }

    #[test]
    fn manual_candidates_roundtrip_with_optional_fields() {
        let file = ManualCandidatesFile {
            schema_version: MANUAL_CANDIDATES_SCHEMA.into(),
            candidates: vec![ManualCandidateRecord {
                id: Some("seed-43".into()),
                title: Some("Complex boundary".into()),
                invariant: Some("data bounds".into()),
                test_targets: vec!["target_a".into(), "target_b".into()],
                do_not_touch: vec!["file_a".into(), "file_b".into()],
                ..ManualCandidateRecord::default()
            }],
        };
        let json = serde_json::to_string(&file)
            .expect("manual_candidates_roundtrip_with_optional_fields json serialize");
        let back: ManualCandidatesFile = serde_json::from_str(&json)
            .expect("manual_candidates_roundtrip_with_optional_fields json deserialize");
        assert_eq!(back, file);
        assert!(back.schema_matches());
        assert_eq!(
            back.candidates[0].test_targets,
            vec!["target_a", "target_b"]
        );
        assert_eq!(back.candidates[0].do_not_touch, vec!["file_a", "file_b"]);
    }

    #[test]
    fn cards_roundtrip_with_review_cards_alias() {
        let raw =
            r#"{"schema_version":"0.2","review_cards":[{"id":"rc-17","title":"slice read"}]}"#;
        let file: CardsFile = serde_json::from_str(raw).unwrap();
        assert_eq!(file.cards.len(), 1);
        assert_eq!(file.cards[0].id.as_deref(), Some("rc-17"));
    }
}
