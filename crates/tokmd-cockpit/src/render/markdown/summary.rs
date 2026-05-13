use std::fmt::Write;

use crate::{CockpitReceipt, format_signed_f64, trend_direction_label};

pub(super) fn render(s: &mut String, receipt: &CockpitReceipt) {
    s.push_str("### Summary\n\n");
    s.push_str("|Metric|Current|\n");
    s.push_str("|---|---:|\n");
    let _ = writeln!(
        s,
        "|Files Changed|{}|",
        receipt.change_surface.files_changed
    );
    let _ = writeln!(s, "|Insertions|{}|", receipt.change_surface.insertions);
    let _ = writeln!(s, "|Deletions|{}|", receipt.change_surface.deletions);
    let _ = writeln!(s, "|Net Lines|{}|", receipt.change_surface.net_lines);
    let _ = writeln!(s, "|Code Health Score|{}/100|", receipt.code_health.score);
    let _ = writeln!(s, "|Risk Score|{}/100|", receipt.risk.score);
    let _ = writeln!(s, "|Test Ratio|{:.2}|", receipt.composition.test_ratio);
    s.push('\n');
}

pub(super) fn render_comparison(s: &mut String, receipt: &CockpitReceipt) {
    let Some(trend) = receipt.trend.as_ref().filter(|t| t.baseline_available) else {
        return;
    };

    s.push_str("### Summary Comparison\n\n");
    s.push_str("|Metric|Baseline|Current|Delta|Change|\n");
    s.push_str("|---|---:|---:|---:|---|\n");

    if let Some(health) = &trend.health {
        let _ = writeln!(
            s,
            "|Health Score|{:.1}|{:.1}|{}|{}|",
            health.previous,
            health.current,
            format_signed_f64(health.delta),
            trend_direction_label(health.direction)
        );
    }
    if let Some(risk) = &trend.risk {
        let _ = writeln!(
            s,
            "|Risk Score|{:.1}|{:.1}|{}|{}|",
            risk.previous,
            risk.current,
            format_signed_f64(risk.delta),
            trend_direction_label(risk.direction)
        );
    }
    if let Some(complexity) = &trend.complexity {
        let cyclomatic_delta = complexity
            .avg_cyclomatic_delta
            .map(format_signed_f64)
            .unwrap_or_else(|| "n/a".to_string());
        let _ = writeln!(
            s,
            "|Avg Cyclomatic|n/a|n/a|{}|{}|",
            cyclomatic_delta,
            trend_direction_label(complexity.direction)
        );
    }

    if let Some(path) = trend.baseline_path.as_deref() {
        let _ = writeln!(s, "\nBaseline: `{}`", path);
    }
    s.push('\n');
}
