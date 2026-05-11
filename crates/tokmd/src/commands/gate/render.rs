//! Result rendering for the `tokmd gate` command.

use anyhow::Result;
use tokmd_gate::RuleLevel;

use super::CombinedGateResult;

/// Print combined results in text format.
pub(super) fn print_text_result(result: &CombinedGateResult) {
    let policy_count = result
        .policy
        .as_ref()
        .map(|p| p.rule_results.len())
        .unwrap_or(0);
    let ratchet_count = result
        .ratchet
        .as_ref()
        .map(|r| r.ratchet_results.len())
        .unwrap_or(0);
    let total_rules = policy_count + ratchet_count;

    if result.passed {
        println!("Gate PASSED ({} rules evaluated)", total_rules);
    } else {
        println!(
            "Gate FAILED: {} error(s), {} warning(s)",
            result.total_errors, result.total_warnings
        );
    }

    println!();

    if let Some(policy) = &result.policy
        && !policy.rule_results.is_empty()
    {
        println!("Policy Rules:");
        for rule_result in &policy.rule_results {
            let status = if rule_result.passed { "PASS" } else { "FAIL" };
            let level = match rule_result.level {
                RuleLevel::Error => "error",
                RuleLevel::Warn => "warn",
            };

            if rule_result.passed {
                println!("  [{}] {} ({})", status, rule_result.name, level);
            } else {
                println!("  [{}] {} ({})", status, rule_result.name, level);
                println!("        Expected: {}", rule_result.expected);
                if let Some(actual) = &rule_result.actual {
                    println!("        Actual: {}", actual);
                }
                if let Some(msg) = &rule_result.message {
                    println!("        Message: {}", msg);
                }
            }
        }
        println!();
    }

    if let Some(ratchet) = &result.ratchet
        && !ratchet.ratchet_results.is_empty()
    {
        println!("Ratchet Rules:");
        for ratchet_result in &ratchet.ratchet_results {
            let status = if ratchet_result.passed {
                "PASS"
            } else {
                "FAIL"
            };
            let level = match ratchet_result.rule.level {
                RuleLevel::Error => "error",
                RuleLevel::Warn => "warn",
            };

            let name = ratchet_result
                .rule
                .description
                .as_deref()
                .unwrap_or(&ratchet_result.rule.pointer);

            println!("  [{}] {} ({})", status, name, level);

            if let Some(baseline) = ratchet_result.baseline_value {
                if let Some(pct) = ratchet_result.change_pct {
                    println!(
                        "        Baseline: {:.2} -> Current: {:.2} ({:+.2}%)",
                        baseline, ratchet_result.current_value, pct
                    );
                } else {
                    println!(
                        "        Baseline: {:.2}, Current: {:.2}",
                        baseline, ratchet_result.current_value
                    );
                }
            } else {
                println!("        Current: {:.2}", ratchet_result.current_value);
            }

            if !ratchet_result.passed {
                println!("        Message: {}", ratchet_result.message);
            }
        }
    }
}

/// Print combined results in JSON format.
pub(super) fn print_json_result(result: &CombinedGateResult) -> Result<()> {
    let json = serde_json::to_string_pretty(&result)?;
    println!("{}", json);
    Ok(())
}
