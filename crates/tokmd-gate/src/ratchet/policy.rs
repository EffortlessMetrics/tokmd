//! Ratchet policy aggregation.

use super::evaluate::evaluate_ratchet_with_options;
use crate::types::{RatchetConfig, RatchetGateResult, RuleLevel};
use serde_json::Value;

/// Evaluate all ratchet rules against baseline and current receipts.
///
/// # Arguments
/// * `config` - The ratchet configuration with rules
/// * `baseline` - The baseline JSON receipt
/// * `current` - The current JSON receipt
///
/// # Returns
/// A `RatchetGateResult` with overall pass/fail and individual results.
pub fn evaluate_ratchet_policy(
    config: &RatchetConfig,
    baseline: &Value,
    current: &Value,
) -> RatchetGateResult {
    let mut ratchet_results = Vec::with_capacity(config.rules.len());

    for rule in &config.rules {
        let result = evaluate_ratchet_with_options(
            rule,
            baseline,
            current,
            config.allow_missing_baseline,
            config.allow_missing_current,
        );

        let failed_error = !result.passed && result.rule.level == RuleLevel::Error;
        ratchet_results.push(result);

        if config.fail_fast && failed_error {
            break;
        }
    }

    RatchetGateResult::from_results(ratchet_results)
}
