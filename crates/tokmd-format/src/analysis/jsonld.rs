//! JSON-LD rendering for analysis receipts.
//!
//! This module owns the Schema.org projection used by
//! `AnalysisFormat::Jsonld` without changing the analysis receipt itself.

use tokmd_analysis_types::AnalysisReceipt;

pub(super) fn render(receipt: &AnalysisReceipt) -> String {
    let name = receipt
        .source
        .inputs
        .first()
        .cloned()
        .unwrap_or_else(|| "tokmd".to_string());
    let totals = receipt.derived.as_ref().map(|d| &d.totals);
    let payload = serde_json::json!({
        "@context": "https://schema.org",
        "@type": "SoftwareSourceCode",
        "name": name,
        "codeLines": totals.map(|t| t.code).unwrap_or(0),
        "commentCount": totals.map(|t| t.comments).unwrap_or(0),
        "lineCount": totals.map(|t| t.lines).unwrap_or(0),
        "fileSize": totals.map(|t| t.bytes).unwrap_or(0),
        "interactionStatistic": {
            "@type": "InteractionCounter",
            "interactionType": "http://schema.org/ReadAction",
            "userInteractionCount": totals.map(|t| t.tokens).unwrap_or(0)
        }
    });
    serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
}
