use tokmd_analysis_types::EffortResults;

const A: f64 = 2.94;
const B: f64 = 1.10;
const C: f64 = 3.67;
const D: f64 = 0.28;

pub fn cocomo2_effort_pm(kloc: f64) -> (f64, f64, f64, f64) {
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

pub fn cocomo2_baseline(kloc_authored: f64) -> EffortResults {
    let kloc = kloc_authored.max(0.0);
    let (effort_p50, schedule_p50, staff_p50, _) = cocomo2_effort_pm(kloc);
    if effort_p50 <= 0.0 || schedule_p50 <= 0.0 {
        return EffortResults {
        effort_pm_p50: 0.0,
        schedule_months_p50: 0.0,
        staff_p50: 0.0,
        effort_pm_low: 0.0,
        effort_pm_p80: 0.0,
        schedule_months_low: 0.0,
        schedule_months_p80: 0.0,
        staff_low: 0.0,
        staff_p80: 0.0,
        };
    }

    let low = 0.18;
    let high = 0.35;
    let effort_pm_low = (effort_p50 * (1.0 - low)).max(0.0);
    let effort_pm_high = effort_p50 * (1.0 + high);
    let schedule_low = (schedule_p50 * (1.0 - (low * 0.45))).max(0.0);
    let schedule_high = schedule_p50 * (1.0 + (high * 0.45));
    let staff_low = if schedule_high > 0.0 {
        effort_pm_low / schedule_high
    } else {
        0.0
    };
    let staff_p80 = if schedule_low > 0.0 {
        effort_pm_high / schedule_low
    } else {
        0.0
    };

    EffortResults {
        effort_pm_p50: effort_p50,
        schedule_months_p50: schedule_p50,
        staff_p50: staff_p50,
        effort_pm_low,
        effort_pm_p80: effort_pm_high,
        schedule_months_low: schedule_low,
        schedule_months_p80: schedule_high,
        staff_low,
        staff_p80,
    }
}
