use tokmd_analysis_types::{EffortConfidence, EffortDriver, EffortResults};

pub fn apply_uncertainty(
    base: EffortResults,
    confidence: &EffortConfidence,
    basis_confidence: f64,
    drivers: &[EffortDriver],
) -> EffortResults {
    if base.effort_pm_p50 <= 0.0 {
        return base;
    }

    let evidence_pressure = drivers
        .iter()
        .map(|driver| driver.weight.max(0.0))
        .sum::<f64>()
        .clamp(0.0, 0.85);

    let confidence_penalty = (1.0 - confidence.data_coverage_pct.unwrap_or(0.0)).clamp(0.0, 0.8);
    let basis_penalty = (1.0 - basis_confidence).clamp(0.0, 0.6);

    let mut width = match confidence.level {
        tokmd_analysis_types::EffortConfidenceLevel::High => 0.10,
        tokmd_analysis_types::EffortConfidenceLevel::Medium => 0.22,
        tokmd_analysis_types::EffortConfidenceLevel::Low => 0.35,
    };
    width += evidence_pressure * 0.06;
    width += confidence_penalty * 0.45;
    width += basis_penalty * 0.35;
    width = width.clamp(0.04, 0.85);

    let effort_low = (base.effort_pm_p50 * (1.0 - width)).max(0.0);
    let effort_p80 = (base.effort_pm_p50 * (1.0 + width)).max(0.0);

    let schedule_width = (width * 0.62).clamp(0.03, 0.70);
    let schedule_low = (base.schedule_months_p50 * (1.0 - schedule_width)).max(0.0);
    let schedule_p80 = (base.schedule_months_p50 * (1.0 + schedule_width)).max(0.0);

    let staff_low = if schedule_p80 > 0.0 {
        effort_low / schedule_p80
    } else {
        0.0
    };
    let staff_p80 = if schedule_low > 0.0 {
        effort_p80 / schedule_low
    } else {
        0.0
    };

    EffortResults {
        effort_pm_p50: base.effort_pm_p50,
        schedule_months_p50: base.schedule_months_p50,
        staff_p50: base.staff_p50,
        effort_pm_low: effort_low,
        effort_pm_p80: effort_p80,
        schedule_months_low: schedule_low,
        schedule_months_p80: schedule_p80,
        staff_low,
        staff_p80,
    }
}
