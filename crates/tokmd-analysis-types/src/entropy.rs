//! Entropy receipt DTOs.
//!
//! These contract types remain re-exported from the crate root to preserve
//! existing `tokmd_analysis_types::...` names.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntropyReport {
    pub suspects: Vec<EntropyFinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntropyFinding {
    pub path: String,
    pub module: String,
    pub entropy_bits_per_byte: f32,
    pub sample_bytes: u32,
    pub class: EntropyClass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntropyClass {
    Low,
    Normal,
    Suspicious,
    High,
}

#[cfg(test)]
mod tests {
    use super::EntropyClass;

    #[test]
    fn entropy_class_serde_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        for variant in [
            EntropyClass::Low,
            EntropyClass::Normal,
            EntropyClass::Suspicious,
            EntropyClass::High,
        ] {
            let json = serde_json::to_string(&variant)?;
            let back: EntropyClass = serde_json::from_str(&json)?;
            assert_eq!(back, variant);
        }
        Ok(())
    }

    #[test]
    fn entropy_class_uses_snake_case() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(
            serde_json::to_string(&EntropyClass::Suspicious)?,
            "\"suspicious\""
        );
        Ok(())
    }
}
