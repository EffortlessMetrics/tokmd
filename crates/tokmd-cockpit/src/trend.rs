use std::path::Path;

use anyhow::Result;
use tokmd_types::cockpit::{
    CockpitReceipt, TrendComparison, TrendDirection, TrendIndicator, TrendMetric,
};

use crate::display::round_pct;

/// Load baseline receipt and compute trend comparison.
pub fn load_and_compute_trend(
    baseline_path: &Path,
    current: &CockpitReceipt,
) -> Result<TrendComparison> {
    // Try to load baseline
    let content = match std::fs::read_to_string(baseline_path) {
        Ok(c) => c,
        Err(_) => {
            return Ok(TrendComparison {
                baseline_available: false,
                baseline_path: Some(baseline_path.to_string_lossy().to_string()),
                ..Default::default()
            });
        }
    };

    let baseline: CockpitReceipt = match serde_json::from_str(&content) {
        Ok(b) => b,
        Err(_) => {
            return Ok(TrendComparison {
                baseline_available: false,
                baseline_path: Some(baseline_path.to_string_lossy().to_string()),
                ..Default::default()
            });
        }
    };

    // Compute health trend
    let health = compute_metric_trend(
        current.code_health.score as f64,
        baseline.code_health.score as f64,
        true, // Higher is better for health
    );

    // Compute risk trend
    let risk = compute_metric_trend(
        current.risk.score as f64,
        baseline.risk.score as f64,
        false, // Lower is better for risk
    );

    // Compute complexity trend indicator
    let complexity = compute_complexity_trend(current, &baseline);

    Ok(TrendComparison {
        baseline_available: true,
        baseline_path: Some(baseline_path.to_string_lossy().to_string()),
        baseline_generated_at_ms: Some(baseline.generated_at_ms),
        health: Some(health),
        risk: Some(risk),
        complexity: Some(complexity),
    })
}

/// Compute trend metric with direction.
pub fn compute_metric_trend(current: f64, previous: f64, higher_is_better: bool) -> TrendMetric {
    let delta = current - previous;
    let delta_pct = if previous != 0.0 {
        (delta / previous) * 100.0
    } else if current != 0.0 {
        100.0
    } else {
        0.0
    };

    // Determine direction based on whether improvement means higher or lower
    let direction = if delta.abs() < 1.0 {
        TrendDirection::Stable
    } else if higher_is_better {
        if delta > 0.0 {
            TrendDirection::Improving
        } else {
            TrendDirection::Degrading
        }
    } else {
        // Lower is better (e.g., risk)
        if delta < 0.0 {
            TrendDirection::Improving
        } else {
            TrendDirection::Degrading
        }
    };

    TrendMetric {
        current,
        previous,
        delta,
        delta_pct: round_pct(delta_pct),
        direction,
    }
}

/// Compute complexity trend indicator.
pub fn compute_complexity_trend(
    current: &CockpitReceipt,
    baseline: &CockpitReceipt,
) -> TrendIndicator {
    // Compare complexity gate results if available
    let current_complexity = current
        .evidence
        .complexity
        .as_ref()
        .map(|c| c.avg_cyclomatic)
        .unwrap_or(0.0);
    let baseline_complexity = baseline
        .evidence
        .complexity
        .as_ref()
        .map(|c| c.avg_cyclomatic)
        .unwrap_or(0.0);

    let delta = current_complexity - baseline_complexity;

    let direction = if delta.abs() < 0.5 {
        TrendDirection::Stable
    } else if delta < 0.0 {
        TrendDirection::Improving
    } else {
        TrendDirection::Degrading
    };

    let summary = match direction {
        TrendDirection::Improving => "Complexity decreased".to_string(),
        TrendDirection::Stable => "Complexity stable".to_string(),
        TrendDirection::Degrading => "Complexity increased".to_string(),
    };

    TrendIndicator {
        direction,
        summary,
        files_increased: 0, // Would require per-file comparison
        files_decreased: 0,
        avg_cyclomatic_delta: Some(round_pct(delta)),
        avg_cognitive_delta: None,
    }
}
