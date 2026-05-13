use std::fmt::Write;

use crate::CockpitReceipt;

pub(super) fn append(s: &mut String, receipt: &CockpitReceipt) {
    let _ = writeln!(s, "### Contracts");
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "- **API changed**: {}",
        yes_no(receipt.contracts.api_changed)
    );
    let _ = writeln!(
        s,
        "- **CLI changed**: {}",
        yes_no(receipt.contracts.cli_changed)
    );
    let _ = writeln!(
        s,
        "- **Schema changed**: {}",
        yes_no(receipt.contracts.schema_changed)
    );
    let _ = writeln!(
        s,
        "- **Breaking indicators**: {}",
        receipt.contracts.breaking_indicators
    );
    let _ = writeln!(s);
}

fn yes_no(value: bool) -> &'static str {
    if value { "Yes" } else { "No" }
}
