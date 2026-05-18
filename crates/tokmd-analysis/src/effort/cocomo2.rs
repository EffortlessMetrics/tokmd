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
        staff_p50,
        effort_pm_low,
        effort_pm_p80: effort_pm_high,
        schedule_months_low: schedule_low,
        schedule_months_p80: schedule_high,
        staff_low,
        staff_p80,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cocomo2_effort_pm_returns_zeros_for_non_positive_kloc() {
        // Saturating exit on `kloc <= 0.0` is the only short-circuit.
        let (e1, s1, p1, _) = cocomo2_effort_pm(0.0);
        assert_eq!((e1, s1, p1), (0.0, 0.0, 0.0));
        let (e2, s2, p2, _) = cocomo2_effort_pm(-5.0);
        assert_eq!((e2, s2, p2), (0.0, 0.0, 0.0));
    }

    #[test]
    fn cocomo2_effort_pm_uses_documented_constants_at_one_kloc() {
        // At 1 KLOC the powers collapse to constants, so we can pin them.
        let (effort, schedule, staff, mirror) = cocomo2_effort_pm(1.0);
        assert!((effort - A).abs() < 1e-9);
        assert!((schedule - C * effort.powf(D)).abs() < 1e-9);
        // staff = effort / schedule
        assert!((staff - effort / schedule).abs() < 1e-9);
        // 4th tuple slot mirrors the p50 effort value.
        assert!((mirror - effort).abs() < 1e-9);
    }

    #[test]
    fn cocomo2_baseline_zero_kloc_returns_all_zeros() {
        let r = cocomo2_baseline(0.0);
        assert_eq!(r.effort_pm_p50, 0.0);
        assert_eq!(r.schedule_months_p50, 0.0);
        assert_eq!(r.staff_p50, 0.0);
        assert_eq!(r.effort_pm_low, 0.0);
        assert_eq!(r.effort_pm_p80, 0.0);
        assert_eq!(r.schedule_months_low, 0.0);
        assert_eq!(r.schedule_months_p80, 0.0);
        assert_eq!(r.staff_low, 0.0);
        assert_eq!(r.staff_p80, 0.0);
    }

    #[test]
    fn cocomo2_baseline_clamps_negative_kloc_to_zero() {
        let r_neg = cocomo2_baseline(-1.0);
        let r_zero = cocomo2_baseline(0.0);
        assert_eq!(r_neg.effort_pm_p50, r_zero.effort_pm_p50);
        assert_eq!(r_neg.schedule_months_p50, r_zero.schedule_months_p50);
    }

    #[test]
    fn cocomo2_baseline_low_p80_envelope_widens_around_p50() {
        let r = cocomo2_baseline(10.0);
        assert!(r.effort_pm_p50 > 0.0);
        assert!(r.effort_pm_low < r.effort_pm_p50);
        assert!(r.effort_pm_p80 > r.effort_pm_p50);
        assert!(r.schedule_months_low < r.schedule_months_p50);
        assert!(r.schedule_months_p80 > r.schedule_months_p50);
        // staff_low uses schedule_high in denominator => smaller than staff_p50,
        // staff_p80 uses schedule_low => larger than staff_p50.
        assert!(r.staff_low < r.staff_p50);
        assert!(r.staff_p80 > r.staff_p50);
    }
}
