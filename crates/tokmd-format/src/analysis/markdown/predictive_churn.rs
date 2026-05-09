//! Predictive churn Markdown rendering.
//!
//! This module owns predictive churn table ordering and empty-state rendering.

use std::fmt::Write;

use super::fmt_f64;
use tokmd_analysis_types::PredictiveChurnReport;

pub(super) fn render_predictive_churn(out: &mut String, churn: &PredictiveChurnReport) {
    out.push_str("## Predictive churn\n\n");
    let mut rows: Vec<_> = churn.per_module.iter().collect();
    rows.sort_by(|a, b| {
        b.1.slope
            .partial_cmp(&a.1.slope)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(b.0))
    });
    if rows.is_empty() {
        out.push_str("- No churn signals detected.\n\n");
    } else {
        out.push_str("|Module|Slope|R²|Recent change|Class|\n");
        out.push_str("|---|---:|---:|---:|---|\n");
        for (module, trend) in rows.into_iter().take(10) {
            let _ = writeln!(
                out,
                "|{}|{}|{}|{}|{:?}|",
                module,
                fmt_f64(trend.slope, 4),
                fmt_f64(trend.r2, 2),
                trend.recent_change,
                trend.classification
            );
        }
        out.push('\n');
    }
}
