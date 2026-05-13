use std::fmt::Write;

use crate::{CockpitReceipt, sparkline};

pub(super) fn render(s: &mut String, receipt: &CockpitReceipt) {
    let Some(ref trend) = receipt.trend else {
        return;
    };

    let _ = writeln!(s, "### Trend");
    let _ = writeln!(s);
    if trend.baseline_available {
        let _ = writeln!(
            s,
            "- **Baseline**: {}",
            trend.baseline_path.as_deref().unwrap_or("N/A")
        );
        if let Some(ref health) = trend.health {
            let _ = writeln!(
                s,
                "- **Health**: {:.1} -> {:.1} {} ({:.1}%, {:?})",
                health.previous,
                health.current,
                sparkline(&[health.previous, health.current]),
                health.delta_pct,
                health.direction
            );
        }
        if let Some(ref risk) = trend.risk {
            let _ = writeln!(
                s,
                "- **Risk**: {:.1} -> {:.1} {} ({:.1}%, {:?})",
                risk.previous,
                risk.current,
                sparkline(&[risk.previous, risk.current]),
                risk.delta_pct,
                risk.direction
            );
        }
        if let Some(ref complexity) = trend.complexity {
            let _ = writeln!(
                s,
                "- **Complexity**: {} ({:?})",
                complexity.summary, complexity.direction
            );
        }
    } else {
        let _ = writeln!(s, "No baseline available for comparison.");
    }
    let _ = writeln!(s);
}
