//! Metric-focused Markdown sections for cockpit receipts.

use std::fmt::Write;

use crate::CockpitReceipt;

pub(super) fn render_change_surface(s: &mut String, receipt: &CockpitReceipt) {
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

pub(super) fn render_composition(s: &mut String, receipt: &CockpitReceipt) {
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

pub(super) fn render_contracts(s: &mut String, receipt: &CockpitReceipt) {
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

pub(super) fn render_code_health(s: &mut String, receipt: &CockpitReceipt) {
    let _ = writeln!(s, "### Code Health");
    let _ = writeln!(s);
    let _ = writeln!(s, "- **Score**: {}/100", receipt.code_health.score);
    let _ = writeln!(s, "- **Grade**: {}", receipt.code_health.grade);
    let _ = writeln!(
        s,
        "- **Large files touched**: {}",
        receipt.code_health.large_files_touched
    );
    let _ = writeln!(
        s,
        "- **Average file size**: {}",
        receipt.code_health.avg_file_size
    );
    let _ = writeln!(
        s,
        "- **Complexity indicator**: {:?}",
        receipt.code_health.complexity_indicator
    );
    if !receipt.code_health.warnings.is_empty() {
        let _ = writeln!(s, "- **Warnings**:");
        for warning in &receipt.code_health.warnings {
            let _ = writeln!(s, "  - {}: {}", warning.path, warning.message);
        }
    }
    let _ = writeln!(s);
}

pub(super) fn render_risk(s: &mut String, receipt: &CockpitReceipt) {
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

fn yes_no(value: bool) -> &'static str {
    if value { "Yes" } else { "No" }
}
