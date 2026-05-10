//! Effort size basis DTOs.
//!
//! These serde-stable contract types remain re-exported from the crate root.

use serde::{Deserialize, Serialize};

use super::EffortConfidenceLevel;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffortSizeBasis {
    pub total_lines: usize,
    pub authored_lines: usize,
    pub generated_lines: usize,
    pub vendored_lines: usize,
    pub kloc_total: f64,
    pub kloc_authored: f64,
    pub generated_pct: f64,
    pub vendored_pct: f64,
    pub classification_confidence: EffortConfidenceLevel,
    pub warnings: Vec<String>,
    pub by_tag: Vec<EffortTagSizeRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffortTagSizeRow {
    pub tag: String,
    pub lines: usize,
    pub authored_lines: usize,
    pub pct_of_total: f64,
}

#[cfg(test)]
mod tests {
    use super::{EffortConfidenceLevel, EffortSizeBasis, EffortTagSizeRow};

    #[test]
    fn effort_size_basis_roundtrip_preserves_tag_rows() {
        let basis = EffortSizeBasis {
            total_lines: 100,
            authored_lines: 80,
            generated_lines: 15,
            vendored_lines: 5,
            kloc_total: 0.1,
            kloc_authored: 0.08,
            generated_pct: 15.0,
            vendored_pct: 5.0,
            classification_confidence: EffortConfidenceLevel::High,
            warnings: vec!["sample".to_string()],
            by_tag: vec![EffortTagSizeRow {
                tag: "generated".to_string(),
                lines: 15,
                authored_lines: 0,
                pct_of_total: 15.0,
            }],
        };

        let json = serde_json::to_string(&basis).unwrap();
        let back: EffortSizeBasis = serde_json::from_str(&json).unwrap();

        assert_eq!(back.total_lines, 100);
        assert_eq!(back.classification_confidence, EffortConfidenceLevel::High);
        assert_eq!(back.by_tag.len(), 1);
        assert_eq!(back.by_tag[0].tag, "generated");
    }
}
