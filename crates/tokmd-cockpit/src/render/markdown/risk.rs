//! Risk section for cockpit Markdown receipts.

use std::fmt::Write;

use crate::CockpitReceipt;

pub(super) fn render(s: &mut String, receipt: &CockpitReceipt) {
    let _ = writeln!(s, "### Risk");
    let _ = writeln!(s);
    let _ = writeln!(s, "- **Level**: {}", receipt.risk.level);
    let _ = writeln!(s, "- **Score**: {}/100", receipt.risk.score);
    if !receipt.risk.hotspots_touched.is_empty() {
        let _ = writeln!(s, "- **Hotspots touched**:");
        for hotspot in &receipt.risk.hotspots_touched {
            let _ = writeln!(s, "  - {}", hotspot);
        }
    }
    if !receipt.risk.bus_factor_warnings.is_empty() {
        let _ = writeln!(s, "- **Bus factor warnings**:");
        for warning in &receipt.risk.bus_factor_warnings {
            let _ = writeln!(s, "  - {}", warning);
        }
    }
    let _ = writeln!(s);
}
