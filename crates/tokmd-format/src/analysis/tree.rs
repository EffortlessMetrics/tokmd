//! Tree rendering for analysis receipts.
//!
//! This module owns the `AnalysisFormat::Tree` projection and preserves the
//! existing fallback when no derived tree is present.

use tokmd_analysis_types::AnalysisReceipt;

pub(super) fn render(receipt: &AnalysisReceipt) -> String {
    receipt
        .derived
        .as_ref()
        .and_then(|d| d.tree.clone())
        .unwrap_or_else(|| "(tree unavailable)".to_string())
}
