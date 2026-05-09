//! SVG badge-style rendering for analysis receipts.
//!
//! This module owns the compact analysis SVG output. It renders receipt data
//! exactly as provided and does not infer missing analysis fields.

use tokmd_analysis_types::AnalysisReceipt;

pub(super) fn render(receipt: &AnalysisReceipt) -> String {
    let (label, value) = if let Some(derived) = &receipt.derived {
        if let Some(ctx) = &derived.context_window {
            ("context".to_string(), format!("{:.1}%", ctx.pct * 100.0))
        } else {
            ("tokens".to_string(), derived.totals.tokens.to_string())
        }
    } else {
        ("tokens".to_string(), "0".to_string())
    };

    let width = 240;
    let height = 32;
    let label_width = 80;
    let value_width = width - label_width;
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{height}\" role=\"img\"><rect width=\"{label_width}\" height=\"{height}\" fill=\"#555\"/><rect x=\"{label_width}\" width=\"{value_width}\" height=\"{height}\" fill=\"#4c9aff\"/><text x=\"{lx}\" y=\"{ty}\" fill=\"#fff\" font-family=\"Verdana\" font-size=\"12\" text-anchor=\"middle\">{label}</text><text x=\"{vx}\" y=\"{ty}\" fill=\"#fff\" font-family=\"Verdana\" font-size=\"12\" text-anchor=\"middle\">{value}</text></svg>",
        width = width,
        height = height,
        label_width = label_width,
        value_width = value_width,
        lx = label_width / 2,
        vx = label_width + value_width / 2,
        ty = 20,
        label = label,
        value = value
    )
}
