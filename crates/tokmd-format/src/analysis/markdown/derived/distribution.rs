//! Distribution and histogram rendering for derived analysis reports.

use std::fmt::Write;

use tokmd_analysis_types::DerivedReport;

use crate::analysis::markdown::{fmt_f64, fmt_pct};

pub(super) fn render_distribution(out: &mut String, derived: &DerivedReport) {
    out.push_str("## Distribution\n\n");
    out.push_str("|Count|Min|Max|Mean|Median|P90|P99|Gini|\n");
    out.push_str("|---:|---:|---:|---:|---:|---:|---:|---:|\n");
    let _ = writeln!(
        out,
        "|{}|{}|{}|{}|{}|{}|{}|{}|\n",
        derived.distribution.count,
        derived.distribution.min,
        derived.distribution.max,
        fmt_f64(derived.distribution.mean, 2),
        fmt_f64(derived.distribution.median, 2),
        fmt_f64(derived.distribution.p90, 2),
        fmt_f64(derived.distribution.p99, 2),
        fmt_f64(derived.distribution.gini, 4)
    );

    render_file_size_histogram(out, derived);
}

fn render_file_size_histogram(out: &mut String, derived: &DerivedReport) {
    out.push_str("## File size histogram\n\n");
    out.push_str("|Bucket|Min|Max|Files|Pct|\n");
    out.push_str("|---|---:|---:|---:|---:|\n");
    for bucket in &derived.histogram {
        let max = bucket
            .max
            .map(|v| v.to_string())
            .unwrap_or_else(|| "∞".to_string());
        let _ = writeln!(
            out,
            "|{}|{}|{}|{}|{}|",
            bucket.label,
            bucket.min,
            max,
            bucket.files,
            fmt_pct(bucket.pct)
        );
    }
    out.push('\n');
}
