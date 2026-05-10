//! Topic analysis receipt DTOs.
//!
//! These contract types remain re-exported from the crate root to preserve
//! existing `tokmd_analysis_types::...` names.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicClouds {
    pub per_module: BTreeMap<String, Vec<TopicTerm>>,
    pub overall: Vec<TopicTerm>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicTerm {
    pub term: String,
    pub score: f64,
    pub tf: u32,
    pub df: u32,
}

#[cfg(test)]
mod tests {
    use super::TopicTerm;

    #[test]
    fn topic_term_serde_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let term = TopicTerm {
            term: "async".into(),
            score: 0.95,
            tf: 10,
            df: 3,
        };
        let json = serde_json::to_string(&term)?;
        let back: TopicTerm = serde_json::from_str(&json)?;
        assert_eq!(back.term, "async");
        assert_eq!(back.tf, 10);
        Ok(())
    }
}
