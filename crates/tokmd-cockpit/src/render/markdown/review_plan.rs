use std::fmt::Write;

use crate::CockpitReceipt;

pub(super) fn append(s: &mut String, receipt: &CockpitReceipt) {
    let _ = writeln!(s, "### Review Plan");
    let _ = writeln!(s);
    if receipt.review_plan.is_empty() {
        let _ = writeln!(s, "No review items.");
    } else {
        for item in &receipt.review_plan {
            let _ = writeln!(s, "- **{}** (priority: {})", item.path, item.priority);
            let _ = writeln!(s, "  - Reason: {}", item.reason);
            if let Some(complexity) = item.complexity {
                let _ = writeln!(s, "  - Complexity: {}", complexity);
            }
            if let Some(lines) = item.lines_changed {
                let _ = writeln!(s, "  - Lines changed: {}", lines);
            }
        }
    }
    let _ = writeln!(s);
}
