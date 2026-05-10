//! License analysis receipt DTOs.
//!
//! These contract types remain re-exported from the crate root to preserve
//! existing `tokmd_analysis_types::...` names.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseReport {
    pub findings: Vec<LicenseFinding>,
    pub effective: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseFinding {
    pub spdx: String,
    pub confidence: f32,
    pub source_path: String,
    pub source_kind: LicenseSourceKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LicenseSourceKind {
    Metadata,
    Text,
}

#[cfg(test)]
mod tests {
    use super::LicenseSourceKind;

    #[test]
    fn license_source_kind_serde_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        for variant in [LicenseSourceKind::Metadata, LicenseSourceKind::Text] {
            let json = serde_json::to_string(&variant)?;
            let back: LicenseSourceKind = serde_json::from_str(&json)?;
            assert_eq!(back, variant);
        }
        Ok(())
    }
}
