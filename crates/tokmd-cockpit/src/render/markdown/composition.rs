use std::fmt::Write;

use crate::CockpitReceipt;

pub(super) fn render(s: &mut String, receipt: &CockpitReceipt) {
    let _ = writeln!(s, "### Composition");
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "- **Code**: {:.1}%",
        receipt.composition.code_pct * 100.0
    );
    let _ = writeln!(
        s,
        "- **Test**: {:.1}%",
        receipt.composition.test_pct * 100.0
    );
    let _ = writeln!(
        s,
        "- **Docs**: {:.1}%",
        receipt.composition.docs_pct * 100.0
    );
    let _ = writeln!(
        s,
        "- **Config**: {:.1}%",
        receipt.composition.config_pct * 100.0
    );
    let _ = writeln!(s, "- **Test ratio**: {:.2}", receipt.composition.test_ratio);
    let _ = writeln!(s);
}
