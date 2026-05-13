//! Markdown rendering for cockpit receipts.

use std::fmt::Write;

use crate::CockpitReceipt;

mod composition;
mod contracts;
mod evidence;
mod health;
mod review_plan;
mod risk;
mod summary;
mod surface;
mod trend;

/// Render receipt as Markdown summary.
pub fn render_markdown(receipt: &CockpitReceipt) -> String {
    let mut s = String::new();

    let _ = writeln!(s, "## Glass Cockpit");
    let _ = writeln!(s);

    summary::render(&mut s, receipt);
    surface::render(&mut s, receipt);
    composition::render(&mut s, receipt);
    contracts::render(&mut s, receipt);
    health::render(&mut s, receipt);
    risk::render(&mut s, receipt);
    evidence::render(&mut s, receipt);
    review_plan::render(&mut s, receipt);
    trend::render(&mut s, receipt);

    s
}
