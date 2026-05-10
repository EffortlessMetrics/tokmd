//! Effort driver DTOs.
//!
//! These serde-stable contract types remain re-exported from the crate root.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffortDriver {
    pub key: String,
    pub label: String,
    pub weight: f64,
    pub direction: EffortDriverDirection,
    pub evidence: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffortDriverDirection {
    Raises,
    Lowers,
    Neutral,
}

#[cfg(test)]
mod tests {
    use super::EffortDriverDirection;

    #[test]
    fn effort_driver_direction_serde_strings_are_stable() {
        assert_eq!(
            serde_json::to_string(&EffortDriverDirection::Raises).unwrap(),
            "\"raises\""
        );
        assert_eq!(
            serde_json::to_string(&EffortDriverDirection::Lowers).unwrap(),
            "\"lowers\""
        );
        assert_eq!(
            serde_json::to_string(&EffortDriverDirection::Neutral).unwrap(),
            "\"neutral\""
        );
    }
}
