pub(crate) const COCOMO81_COEFFICIENTS: (f64, f64, f64, f64) = (2.4, 1.05, 2.5, 0.38);

pub(crate) fn cocomo81_effort_pm(kloc: f64) -> (f64, f64, f64, f64) {
    if kloc <= 0.0 {
        return (0.0, 0.0, 0.0, 0.0);
    }

    let (a, b, c, d) = COCOMO81_COEFFICIENTS;
    let effort_pm = a * kloc.powf(b);
    let schedule_months = if effort_pm <= 0.0 {
        0.0
    } else {
        c * effort_pm.powf(d)
    };
    let staff = if schedule_months > 0.0 {
        effort_pm / schedule_months
    } else {
        0.0
    };

    (effort_pm, schedule_months, staff, effort_pm)
}
