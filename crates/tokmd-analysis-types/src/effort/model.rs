//! Effort estimation model enum DTOs.
//!
//! These serde-stable contract types remain re-exported from the crate root.

use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EffortModel {
    Cocomo81Basic,
    Cocomo2Early,
    Ensemble,
}

impl fmt::Display for EffortModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cocomo81Basic => f.write_str("cocomo81-basic"),
            Self::Cocomo2Early => f.write_str("cocomo2-early"),
            Self::Ensemble => f.write_str("ensemble"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::EffortModel;

    #[test]
    fn effort_model_display_strings_are_stable() {
        assert_eq!(EffortModel::Cocomo81Basic.to_string(), "cocomo81-basic");
        assert_eq!(EffortModel::Cocomo2Early.to_string(), "cocomo2-early");
        assert_eq!(EffortModel::Ensemble.to_string(), "ensemble");
    }
}
