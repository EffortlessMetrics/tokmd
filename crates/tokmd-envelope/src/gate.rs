//! Evidence gate DTOs embedded in sensor reports.

use crate::Verdict;
use serde::{Deserialize, Serialize};

/// Evidence gate results section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateResults {
    /// Overall gate status.
    pub status: Verdict,
    /// Individual gate items.
    pub items: Vec<GateItem>,
}

/// Individual gate item in the gates section.
///
/// # Examples
///
/// ```
/// use tokmd_envelope::{GateItem, Verdict};
///
/// let gate = GateItem::new("coverage", Verdict::Pass)
///     .with_threshold(80.0, 85.5)
///     .with_source("ci_artifact");
/// assert_eq!(gate.id, "coverage");
/// assert_eq!(gate.actual, Some(85.5));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateItem {
    /// Gate identifier (e.g., "mutation", "diff_coverage").
    pub id: String,
    /// Gate status.
    pub status: Verdict,
    /// Threshold value (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold: Option<f64>,
    /// Actual measured value (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual: Option<f64>,
    /// Reason for the status (especially for pending/fail).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Data source (e.g., "ci_artifact", "computed").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Path to the source artifact (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact_path: Option<String>,
}

impl GateResults {
    /// Create a new gate results section.
    pub fn new(status: Verdict, items: Vec<GateItem>) -> Self {
        Self { status, items }
    }
}

impl GateItem {
    /// Create a new gate item with required fields.
    pub fn new(id: impl Into<String>, status: Verdict) -> Self {
        Self {
            id: id.into(),
            status,
            threshold: None,
            actual: None,
            reason: None,
            source: None,
            artifact_path: None,
        }
    }

    /// Attach threshold and measured values to this gate item.
    pub fn with_threshold(mut self, threshold: f64, actual: f64) -> Self {
        self.threshold = Some(threshold);
        self.actual = Some(actual);
        self
    }

    /// Add a reason to the gate item.
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    /// Add a source to the gate item.
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Add an artifact path to the gate item.
    pub fn with_artifact_path(mut self, path: impl Into<String>) -> Self {
        self.artifact_path = Some(path.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::{GateItem, GateResults};
    use crate::Verdict;

    #[test]
    fn gate_results_new_preserves_status_and_items() {
        let gate = GateItem::new("coverage", Verdict::Pass);
        let results = GateResults::new(Verdict::Pass, vec![gate]);

        assert_eq!(results.status, Verdict::Pass);
        assert_eq!(results.items.len(), 1);
        assert_eq!(results.items[0].id, "coverage");
    }

    #[test]
    fn gate_results_serde_roundtrip() {
        let results = GateResults::new(
            Verdict::Warn,
            vec![
                GateItem::new("diff_coverage", Verdict::Warn)
                    .with_threshold(0.8, 0.72)
                    .with_reason("Below threshold")
                    .with_source("ci_artifact")
                    .with_artifact_path("coverage/lcov.info"),
            ],
        );

        let json = serde_json::to_string(&results).unwrap();
        let back: GateResults = serde_json::from_str(&json).unwrap();

        assert_eq!(back.status, Verdict::Warn);
        assert_eq!(back.items.len(), 1);
        assert_eq!(back.items[0].id, "diff_coverage");
        assert_eq!(back.items[0].status, Verdict::Warn);
        assert_eq!(back.items[0].threshold, Some(0.8));
        assert_eq!(back.items[0].actual, Some(0.72));
        assert_eq!(back.items[0].reason.as_deref(), Some("Below threshold"));
        assert_eq!(back.items[0].source.as_deref(), Some("ci_artifact"));
        assert_eq!(
            back.items[0].artifact_path.as_deref(),
            Some("coverage/lcov.info")
        );
    }

    #[test]
    fn gate_item_omits_optional_fields_when_none() {
        let item = GateItem::new("mutation", Verdict::Pending);
        let json = serde_json::to_value(&item).unwrap();

        assert_eq!(json["id"], "mutation");
        assert_eq!(json["status"], "pending");
        assert!(json.get("threshold").is_none());
        assert!(json.get("actual").is_none());
        assert!(json.get("reason").is_none());
        assert!(json.get("source").is_none());
        assert!(json.get("artifact_path").is_none());
    }

    #[test]
    fn gate_item_builder_fields() {
        let gate = GateItem::new("diff_coverage", Verdict::Warn)
            .with_threshold(0.8, 0.72)
            .with_reason("Below threshold")
            .with_source("ci_artifact")
            .with_artifact_path("coverage/lcov.info");

        assert_eq!(gate.id, "diff_coverage");
        assert_eq!(gate.status, Verdict::Warn);
        assert_eq!(gate.threshold, Some(0.8));
        assert_eq!(gate.actual, Some(0.72));
        assert_eq!(gate.reason.as_deref(), Some("Below threshold"));
        assert_eq!(gate.source.as_deref(), Some("ci_artifact"));
        assert_eq!(gate.artifact_path.as_deref(), Some("coverage/lcov.info"));
    }
}
