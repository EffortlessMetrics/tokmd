//! Section-marker Markdown rendering for cockpit receipts.

use std::fmt::Write;

use crate::CockpitReceipt;

/// Render receipt as sectioned output.
pub fn render_sections(receipt: &CockpitReceipt) -> String {
    let mut s = String::new();

    let _ = writeln!(s, "<!-- SECTION:COCKPIT -->");
    let _ = writeln!(s);
    let _ = writeln!(s, "## Glass Cockpit");
    let _ = writeln!(s);
    let _ = writeln!(s, "**Base**: {}", receipt.base_ref);
    let _ = writeln!(s, "**Head**: {}", receipt.head_ref);
    let _ = writeln!(s);
    let _ = writeln!(s, "**Change Surface**:");
    let _ = writeln!(s, "- Files: {}", receipt.change_surface.files_changed);
    let _ = writeln!(s, "- Insertions: {}", receipt.change_surface.insertions);
    let _ = writeln!(s, "- Deletions: {}", receipt.change_surface.deletions);
    let _ = writeln!(s);
    let _ = writeln!(s, "**Composition**:");
    let _ = writeln!(s, "- Code: {:.1}%", receipt.composition.code_pct * 100.0);
    let _ = writeln!(s, "- Test: {:.1}%", receipt.composition.test_pct * 100.0);
    let _ = writeln!(s, "- Docs: {:.1}%", receipt.composition.docs_pct * 100.0);
    let _ = writeln!(
        s,
        "- Config: {:.1}%",
        receipt.composition.config_pct * 100.0
    );
    let _ = writeln!(s);
    let _ = writeln!(s, "**Contracts**:");
    let _ = writeln!(
        s,
        "- API: {}",
        if receipt.contracts.api_changed {
            "Yes"
        } else {
            "No"
        }
    );
    let _ = writeln!(
        s,
        "- CLI: {}",
        if receipt.contracts.cli_changed {
            "Yes"
        } else {
            "No"
        }
    );
    let _ = writeln!(
        s,
        "- Schema: {}",
        if receipt.contracts.schema_changed {
            "Yes"
        } else {
            "No"
        }
    );
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "**Health**: {}/100 ({})",
        receipt.code_health.score, receipt.code_health.grade
    );
    let _ = writeln!(
        s,
        "**Risk**: {} ({}/100)",
        receipt.risk.level, receipt.risk.score
    );
    let _ = writeln!(s);
    let _ = writeln!(s, "<!-- SECTION:REVIEW_PLAN -->");
    let _ = writeln!(s);
    let _ = writeln!(s, "## Review Plan");
    let _ = writeln!(s);
    if receipt.review_plan.is_empty() {
        let _ = writeln!(s, "No review items.");
    } else {
        for item in &receipt.review_plan {
            let _ = writeln!(s, "- {} (priority: {})", item.path, item.priority);
        }
    }
    let _ = writeln!(s);
    let _ = writeln!(s, "<!-- SECTION:RECEIPTS -->");
    let _ = writeln!(s);
    let _ = writeln!(s, "## Receipts");
    let _ = writeln!(s);
    let _ = writeln!(s, "Full receipt data available in JSON format.");
    let _ = writeln!(s);

    s
}
