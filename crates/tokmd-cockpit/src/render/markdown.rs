//! Markdown rendering for cockpit receipts.

use crate::CockpitReceipt;

mod change_surface;
mod code_health;
mod composition;
mod contracts;
mod evidence_gates;
mod review_plan;
mod risk;
mod summary;
mod trend;

/// Render receipt as Markdown summary.
pub fn render_markdown(receipt: &CockpitReceipt) -> String {
    let mut s = String::new();

    summary::append_title(&mut s);
    summary::append_summary_table(&mut s, receipt);
    summary::append_summary_comparison(&mut s, receipt);
    change_surface::append(&mut s, receipt);
    composition::append(&mut s, receipt);
    contracts::append(&mut s, receipt);
    code_health::append(&mut s, receipt);
    risk::append(&mut s, receipt);
    evidence_gates::append(&mut s, receipt);
    review_plan::append(&mut s, receipt);
    trend::append(&mut s, receipt);

    s
}
