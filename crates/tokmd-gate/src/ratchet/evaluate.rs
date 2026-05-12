//! Single-rule ratchet evaluation.

use super::change::percentage_change;
use crate::numeric::value_to_f64;
use crate::pointer::resolve_pointer;
use crate::types::{RatchetResult, RatchetRule};
use serde_json::Value;

/// Evaluate a single ratchet rule against baseline and current values.
///
/// This is a convenience wrapper around [`evaluate_ratchet_with_options`] with
/// strict missing value handling (fails if baseline or current values are missing).
///
/// # Arguments
/// * `rule` - The ratchet rule to evaluate
/// * `baseline` - The baseline JSON receipt to compare against
/// * `current` - The current JSON receipt to check
///
/// # Returns
/// A `RatchetResult` with pass/fail status and detailed information.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn evaluate_ratchet(
    rule: &RatchetRule,
    baseline: &Value,
    current: &Value,
) -> RatchetResult {
    evaluate_ratchet_with_options(rule, baseline, current, false, false)
}

/// Evaluate a ratchet rule with configurable missing value handling.
///
/// # Arguments
/// * `rule` - The ratchet rule to evaluate
/// * `baseline` - The baseline JSON receipt
/// * `current` - The current JSON receipt
/// * `allow_missing_baseline` - Treat missing baseline as pass
/// * `allow_missing_current` - Treat missing current value as pass
///
/// # Returns
/// A `RatchetResult` with pass/fail status.
pub(super) fn evaluate_ratchet_with_options(
    rule: &RatchetRule,
    baseline: &Value,
    current: &Value,
    allow_missing_baseline: bool,
    allow_missing_current: bool,
) -> RatchetResult {
    let baseline_resolved = resolve_pointer(baseline, &rule.pointer);
    let current_resolved = resolve_pointer(current, &rule.pointer);

    // Handle missing current value.
    let current_value = match current_resolved.and_then(value_to_f64) {
        Some(v) => v,
        None => {
            if allow_missing_current {
                return RatchetResult {
                    rule: rule.clone(),
                    passed: true,
                    baseline_value: baseline_resolved.and_then(value_to_f64),
                    current_value: f64::NAN,
                    change_pct: None,
                    message: format!(
                        "Current value not found at pointer '{}' (allowed)",
                        rule.pointer
                    ),
                };
            } else {
                return RatchetResult {
                    rule: rule.clone(),
                    passed: false,
                    baseline_value: baseline_resolved.and_then(value_to_f64),
                    current_value: f64::NAN,
                    change_pct: None,
                    message: format!(
                        "Current value not found or not numeric at pointer '{}'",
                        rule.pointer
                    ),
                };
            }
        }
    };

    // Extract baseline value.
    let baseline_value = baseline_resolved.and_then(value_to_f64);

    let change_pct = percentage_change(baseline_value, current_value);

    // Check max_value constraint (absolute ceiling).
    if let Some(max_val) = rule.max_value
        && current_value > max_val
    {
        return RatchetResult {
            rule: rule.clone(),
            passed: false,
            baseline_value,
            current_value,
            change_pct,
            message: format!(
                "Current value {} exceeds maximum allowed value {}",
                current_value, max_val
            ),
        };
    }

    // Check max_increase_pct constraint (relative to baseline).
    if let Some(max_inc_pct) = rule.max_increase_pct {
        match baseline_value {
            Some(_bv) => {
                let pct = change_pct.unwrap_or(0.0);
                if pct > max_inc_pct {
                    return RatchetResult {
                        rule: rule.clone(),
                        passed: false,
                        baseline_value,
                        current_value,
                        change_pct,
                        message: format!(
                            "Increase of {:.2}% exceeds maximum allowed increase of {:.2}%",
                            pct, max_inc_pct
                        ),
                    };
                }
            }
            None => {
                // No baseline value - can't evaluate percentage increase.
                if allow_missing_baseline {
                    let desc = rule
                        .description
                        .as_deref()
                        .unwrap_or("Ratchet check passed (no baseline)");
                    return RatchetResult {
                        rule: rule.clone(),
                        passed: true,
                        baseline_value: None,
                        current_value,
                        change_pct: None,
                        message: format!("{}: current value = {}", desc, current_value),
                    };
                } else {
                    return RatchetResult {
                        rule: rule.clone(),
                        passed: false,
                        baseline_value: None,
                        current_value,
                        change_pct: None,
                        message: format!(
                            "Baseline value not found at pointer '{}', cannot evaluate percentage increase",
                            rule.pointer
                        ),
                    };
                }
            }
        }
    }

    // All checks passed.
    let desc = rule
        .description
        .as_deref()
        .unwrap_or("Ratchet check passed");

    let message = match (baseline_value, change_pct) {
        (Some(bv), Some(pct)) => format!("{}: {} -> {} ({:+.2}%)", desc, bv, current_value, pct),
        _ => format!("{}: current value = {}", desc, current_value),
    };

    RatchetResult {
        rule: rule.clone(),
        passed: true,
        baseline_value,
        current_value,
        change_pct,
        message,
    }
}
