use std::fmt::Write;

use crate::CockpitReceipt;

pub(super) fn append(s: &mut String, receipt: &CockpitReceipt) {
    let _ = writeln!(s, "### Change Surface");
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "- **Files changed**: {}",
        receipt.change_surface.files_changed
    );
    let _ = writeln!(s, "- **Insertions**: {}", receipt.change_surface.insertions);
    let _ = writeln!(s, "- **Deletions**: {}", receipt.change_surface.deletions);
    let _ = writeln!(s, "- **Net lines**: {}", receipt.change_surface.net_lines);
    let _ = writeln!(
        s,
        "- **Churn velocity**: {:.1}",
        receipt.change_surface.churn_velocity
    );
    let _ = writeln!(s);
}
