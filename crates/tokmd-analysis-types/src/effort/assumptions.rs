//! Effort assumption DTOs.
//!
//! These serde-stable contract types remain re-exported from the crate root.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffortAssumptions {
    pub notes: Vec<String>,
    pub overrides: BTreeMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::EffortAssumptions;
    use std::collections::BTreeMap;

    #[test]
    fn effort_assumptions_roundtrip_preserves_overrides() {
        let assumptions = EffortAssumptions {
            notes: vec!["Generated code excluded".to_string()],
            overrides: BTreeMap::from([
                ("model".to_string(), "cocomo2".to_string()),
                ("risk".to_string(), "nominal".to_string()),
            ]),
        };

        let json = serde_json::to_string(&assumptions).unwrap();
        let back: EffortAssumptions = serde_json::from_str(&json).unwrap();

        assert_eq!(back.notes, assumptions.notes);
        assert_eq!(back.overrides, assumptions.overrides);
    }
}
