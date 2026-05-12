//! Maintainability index formula and grading helpers.

use tokmd_analysis_types::MaintainabilityIndex;

/// Compute maintainability index using simplified or full SEI formula.
///
/// Simplified:
/// MI = 171 - 0.23 * CC - 16.2 * ln(LOC)
///
/// Full (when Halstead volume is available and positive):
/// MI = 171 - 5.2 * ln(V) - 0.23 * CC - 16.2 * ln(LOC)
pub(crate) fn compute_maintainability_index(
    avg_cyclomatic: f64,
    avg_loc: f64,
    halstead_volume: Option<f64>,
) -> Option<MaintainabilityIndex> {
    if avg_loc <= 0.0 {
        return None;
    }

    let avg_loc = round_f64(avg_loc, 2);
    let (raw_score, avg_halstead_volume) = match halstead_volume {
        Some(volume) if volume > 0.0 => (
            171.0 - 5.2 * volume.ln() - 0.23 * avg_cyclomatic - 16.2 * avg_loc.ln(),
            Some(volume),
        ),
        _ => (171.0 - 0.23 * avg_cyclomatic - 16.2 * avg_loc.ln(), None),
    };

    let score = round_f64(raw_score.max(0.0), 2);
    Some(MaintainabilityIndex {
        score,
        avg_cyclomatic,
        avg_loc,
        avg_halstead_volume,
        grade: grade_for_score(score).to_string(),
    })
}

fn grade_for_score(score: f64) -> &'static str {
    if score >= 85.0 {
        "A"
    } else if score >= 65.0 {
        "B"
    } else {
        "C"
    }
}

fn round_f64(val: f64, decimals: u32) -> f64 {
    let factor = 10f64.powi(decimals as i32);
    (val * factor).round() / factor
}

#[cfg(test)]
mod tests {
    use super::compute_maintainability_index;

    #[test]
    fn compute_simplified_index() {
        let mi = compute_maintainability_index(10.0, 100.0, None).expect("mi");
        assert!((mi.score - 94.1).abs() < f64::EPSILON);
        assert_eq!(mi.grade, "A");
        assert_eq!(mi.avg_halstead_volume, None);
    }

    #[test]
    fn compute_full_index_with_halstead() {
        let mi = compute_maintainability_index(10.0, 100.0, Some(200.0)).expect("mi");
        assert!((mi.score - 66.54).abs() < f64::EPSILON);
        assert_eq!(mi.grade, "B");
        assert_eq!(mi.avg_halstead_volume, Some(200.0));
    }
}
