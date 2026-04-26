use tokmd_analysis_types::{EffortConfidence, EffortDriver, EffortResults};

pub fn apply_monte_carlo(
    base: &EffortResults,
    _drivers: &[EffortDriver],
    _confidence: &EffortConfidence,
    _basis_confidence: f64,
    iterations: usize,
    _seed: Option<u64>,
) -> EffortResults {
    if iterations == 0 {
        return base.clone();
    }

    base.clone()
}
