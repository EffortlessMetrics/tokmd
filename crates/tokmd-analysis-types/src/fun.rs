//! Fun and novelty analysis receipt DTOs.
//!
//! These contract types remain re-exported from the crate root to preserve
//! existing `tokmd_analysis_types::...` names.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunReport {
    pub eco_label: Option<EcoLabel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcoLabel {
    pub score: f64,
    pub label: String,
    pub bytes: u64,
    pub notes: String,
}

#[cfg(test)]
mod tests {
    use super::EcoLabel;

    #[test]
    fn eco_label_serde_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let label = EcoLabel {
            score: 85.0,
            label: "A".into(),
            bytes: 1000,
            notes: "Good".into(),
        };
        let json = serde_json::to_string(&label)?;
        let back: EcoLabel = serde_json::from_str(&json)?;
        assert_eq!(back.label, "A");
        assert_eq!(back.bytes, 1000);
        Ok(())
    }
}
