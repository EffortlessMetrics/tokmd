//! Markdown rendering for cockpit receipts.

use crate::CockpitReceipt;

mod evidence;
mod metrics;
mod review_plan;
mod summary;
mod trend;

/// Render receipt as Markdown summary.
pub fn render_markdown(receipt: &CockpitReceipt) -> String {
    let mut s = String::new();

    summary::render_header(&mut s);
    summary::render_summary(&mut s, receipt);
    summary::render_summary_comparison(&mut s, receipt);
    metrics::render_change_surface(&mut s, receipt);
    metrics::render_composition(&mut s, receipt);
    metrics::render_contracts(&mut s, receipt);
    metrics::render_code_health(&mut s, receipt);
    metrics::render_risk(&mut s, receipt);
    evidence::render_evidence_gates(&mut s, receipt);
    review_plan::render_review_plan(&mut s, receipt);
    trend::render_trend(&mut s, receipt);

    s
}
