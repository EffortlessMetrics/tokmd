//! Metric-card rendering for analysis HTML reports.

use super::format::{format_number, format_pct};
use tokmd_analysis_types::AnalysisReceipt;

pub(super) fn build_metrics_cards(receipt: &AnalysisReceipt) -> String {
    let mut cards = String::new();

    if let Some(derived) = &receipt.derived {
        let metrics = [
            ("Files", derived.totals.files.to_string()),
            ("Lines", format_number(derived.totals.lines)),
            ("Code", format_number(derived.totals.code)),
            ("Tokens", format_number(derived.totals.tokens)),
            ("Doc%", format_pct(derived.doc_density.total.ratio)),
        ];

        for (label, value) in metrics {
            cards.push_str(&format!(
                r#"<div class="metric-card"><span class="value">{}</span><span class="label">{}</span></div>"#,
                value, label
            ));
        }

        if let Some(ctx) = &derived.context_window {
            cards.push_str(&format!(
                r#"<div class="metric-card"><span class="value">{}</span><span class="label">Context Fit</span></div>"#,
                format_pct(ctx.pct)
            ));
        }
    }

    cards
}
