use tokmd_analysis_types::EffortResults;

const A: f64 = 2.4;
const B: f64 = 1.05;
const C: f64 = 2.5;
const D: f64 = 0.38;

pub fn coefficients() -> (f64, f64, f64, f64) {
    (A, B, C, D)
}

pub fn cocomo81_effort_pm(kloc: f64) -> (f64, f64, f64, f64) {
    if kloc <= 0.0 {
        return (0.0, 0.0, 0.0, 0.0);
    }

    let effort_pm = A * kloc.powf(B);
    let schedule_months = if effort_pm <= 0.0 {
        0.0
    } else {
        C * effort_pm.powf(D)
    };
    let staff = if schedule_months > 0.0 {
        effort_pm / schedule_months
    } else {
        0.0
    };

    (effort_pm, schedule_months, staff, effort_pm)
}

pub fn estimate_with_factors(
    kloc: f64,
    low: f64,
    high: f64,
) -> (f64, f64, f64, f64, f64, f64, f64, f64, f64) {
    let (p50, p50_schedule, _, _) = cocomo81_effort_pm(kloc);
    if p50 <= 0.0 || p50_schedule <= 0.0 {
        return (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    }

    let low = low.abs();
    let high = high.abs();
    let effort_pm_low = (p50 * (1.0 - low)).max(0.0);
    let effort_pm_high = p50 * (1.0 + high);

    let schedule_months_low =
        (p50_schedule * (1.0 - (low * 0.45).clamp(0.0, 0.9))).max(0.0);
    let schedule_months_high = p50_schedule * (1.0 + (high * 0.45).clamp(0.0, 0.9));

    let staff_low = if schedule_months_high > 0.0 {
        effort_pm_low / schedule_months_high
    } else {
        0.0
    };
    let staff_p50 = if p50_schedule > 0.0 { p50 / p50_schedule } else { 0.0 };
    let staff_high = if schedule_months_low > 0.0 {
        effort_pm_high / schedule_months_low
    } else {
        0.0
    };

    (
        effort_pm_low,
        p50,
        effort_pm_high,
        schedule_months_low,
        p50_schedule,
        schedule_months_high,
        staff_low,
        staff_p50,
        staff_high,
    )
}

pub fn cocomo81_baseline(kloc_authored: f64) -> EffortResults {
    let kloc = kloc_authored.max(0.0);
    let (_, schedule_p50, _, _) = cocomo81_effort_pm(kloc);
    let (effort_pm_low, effort_pm_p50, effort_pm_p80, schedule_low, _, schedule_high, staff_low, staff_p50, staff_p80) =
        estimate_with_factors(kloc, 0.15, 0.30);

    EffortResults {
        effort_pm_p50,
        schedule_months_p50: schedule_p50,
        staff_p50,
        effort_pm_low,
        effort_pm_p80,
        schedule_months_low: schedule_low,
        schedule_months_p80: schedule_high,
        staff_low,
        staff_p80,
    }
}
