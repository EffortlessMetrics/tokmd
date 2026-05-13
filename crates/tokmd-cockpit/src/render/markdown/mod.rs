//! Markdown rendering for cockpit receipts.

use std::fmt::Write;

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

    let _ = writeln!(s, "## Glass Cockpit");
    let _ = writeln!(s);

    summary::render(&mut s, receipt);
    summary::render_comparison(&mut s, receipt);
    change_surface::render(&mut s, receipt);
    composition::render(&mut s, receipt);
    contracts::render(&mut s, receipt);
    code_health::render(&mut s, receipt);
    risk::render(&mut s, receipt);
    evidence_gates::render(&mut s, receipt);
    review_plan::render(&mut s, receipt);
    trend::render(&mut s, receipt);

    s
}
