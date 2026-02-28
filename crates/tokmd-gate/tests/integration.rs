//! Integration tests for tokmd-gate.
//!
//! Full gate evaluation workflows simulating real CI/CD policy scenarios.

use serde_json::json;
use tokmd_gate::{
    evaluate_policy, evaluate_ratchet_policy, PolicyConfig, PolicyRule, RatchetConfig, RatchetRule,
    RuleLevel, RuleOperator,
};

// ============================================================================
// Helpers
// ============================================================================

/// A realistic analysis receipt structure.
fn sample_receipt() -> serde_json::Value {
    json!({
        "schema_version": 2,
        "generator": "tokmd",
        "derived": {
            "totals": {
                "files": 150,
                "code": 12000,
                "comments": 2000,
                "blanks": 1500,
                "tokens": 280000
            },
            "density": {
                "comment_ratio": 0.143,
                "blank_ratio": 0.097
            },
            "distribution": {
                "top_language": "Rust",
                "language_count": 5
            },
            "cocomo": {
                "effort_months": 3.2,
                "cost_usd": 48000
            }
        },
        "languages": [
            {"name": "Rust", "code": 8000, "comments": 1500, "blanks": 1000},
            {"name": "Python", "code": 2000, "comments": 300, "blanks": 200},
            {"name": "TOML", "code": 500, "comments": 0, "blanks": 100},
            {"name": "Markdown", "code": 1000, "comments": 0, "blanks": 150},
            {"name": "YAML", "code": 500, "comments": 200, "blanks": 50}
        ],
        "metadata": {
            "license": "MIT",
            "has_readme": true,
            "has_changelog": true,
            "ci_provider": "github-actions"
        },
        "tags": ["rust", "cli", "analysis", "linting"]
    })
}

/// A baseline receipt for ratchet comparisons.
fn baseline_receipt() -> serde_json::Value {
    json!({
        "derived": {
            "totals": {
                "files": 140,
                "code": 11000,
                "tokens": 260000
            },
            "density": {
                "comment_ratio": 0.15
            },
            "cocomo": {
                "effort_months": 2.9
            }
        },
        "complexity": {
            "avg_cyclomatic": 4.2,
            "max_cyclomatic": 18,
            "avg_cognitive": 3.8
        }
    })
}

fn current_receipt() -> serde_json::Value {
    json!({
        "derived": {
            "totals": {
                "files": 150,
                "code": 12000,
                "tokens": 280000
            },
            "density": {
                "comment_ratio": 0.143
            },
            "cocomo": {
                "effort_months": 3.2
            }
        },
        "complexity": {
            "avg_cyclomatic": 4.5,
            "max_cyclomatic": 20,
            "avg_cognitive": 4.0
        }
    })
}

// ============================================================================
// Workflow: Standard CI gate policy
// ============================================================================

#[test]
fn workflow_standard_ci_gate_all_pass() {
    let receipt = sample_receipt();
    let policy = PolicyConfig {
        rules: vec![
            PolicyRule {
                name: "max_token_budget".into(),
                pointer: "/derived/totals/tokens".into(),
                op: RuleOperator::Lte,
                value: Some(json!(500_000)),
                values: None,
                negate: false,
                level: RuleLevel::Error,
                message: Some("Token budget exceeded for LLM context".into()),
            },
            PolicyRule {
                name: "min_code_lines".into(),
                pointer: "/derived/totals/code".into(),
                op: RuleOperator::Gte,
                value: Some(json!(100)),
                values: None,
                negate: false,
                level: RuleLevel::Error,
                message: None,
            },
            PolicyRule {
                name: "has_readme".into(),
                pointer: "/metadata/has_readme".into(),
                op: RuleOperator::Eq,
                value: Some(json!(true)),
                values: None,
                negate: false,
                level: RuleLevel::Error,
                message: Some("README is required".into()),
            },
            PolicyRule {
                name: "approved_license".into(),
                pointer: "/metadata/license".into(),
                op: RuleOperator::In,
                value: None,
                values: Some(vec![
                    json!("MIT"),
                    json!("Apache-2.0"),
                    json!("BSD-3-Clause"),
                ]),
                negate: false,
                level: RuleLevel::Error,
                message: Some("License must be OSI-approved".into()),
            },
            PolicyRule {
                name: "has_rust_tag".into(),
                pointer: "/tags".into(),
                op: RuleOperator::Contains,
                value: Some(json!("rust")),
                values: None,
                negate: false,
                level: RuleLevel::Warn,
                message: None,
            },
        ],
        fail_fast: false,
        allow_missing: false,
    };

    let result = evaluate_policy(&receipt, &policy);
    assert!(result.passed, "All rules should pass: {:#?}", result);
    assert_eq!(result.errors, 0);
    assert_eq!(result.warnings, 0);
    assert_eq!(result.rule_results.len(), 5);
    // Verify all individual results passed
    for rr in &result.rule_results {
        assert!(rr.passed, "Rule '{}' should have passed", rr.name);
    }
}

#[test]
fn workflow_standard_ci_gate_some_fail() {
    let receipt = json!({
        "derived": {
            "totals": {
                "tokens": 750_000,
                "code": 50
            }
        },
        "metadata": {
            "license": "AGPL-3.0",
            "has_readme": false
        }
    });

    let policy = PolicyConfig {
        rules: vec![
            PolicyRule {
                name: "max_token_budget".into(),
                pointer: "/derived/totals/tokens".into(),
                op: RuleOperator::Lte,
                value: Some(json!(500_000)),
                values: None,
                negate: false,
                level: RuleLevel::Error,
                message: Some("Token budget exceeded".into()),
            },
            PolicyRule {
                name: "min_code_lines".into(),
                pointer: "/derived/totals/code".into(),
                op: RuleOperator::Gte,
                value: Some(json!(100)),
                values: None,
                negate: false,
                level: RuleLevel::Error,
                message: Some("Not enough code".into()),
            },
            PolicyRule {
                name: "has_readme".into(),
                pointer: "/metadata/has_readme".into(),
                op: RuleOperator::Eq,
                value: Some(json!(true)),
                values: None,
                negate: false,
                level: RuleLevel::Warn,
                message: Some("README is recommended".into()),
            },
        ],
        fail_fast: false,
        allow_missing: false,
    };

    let result = evaluate_policy(&receipt, &policy);
    assert!(!result.passed);
    assert_eq!(result.errors, 2);   // tokens + code
    assert_eq!(result.warnings, 1); // readme
    assert_eq!(result.rule_results.len(), 3);
}

// ============================================================================
// Workflow: TOML-driven policy evaluation
// ============================================================================

#[test]
fn workflow_toml_driven_policy() {
    let toml = r#"
fail_fast = false
allow_missing = false

[[rules]]
name = "max_tokens"
pointer = "/derived/totals/tokens"
op = "lte"
value = 500000
level = "error"
message = "Token budget exceeded"

[[rules]]
name = "top_language"
pointer = "/derived/distribution/top_language"
op = "eq"
value = "Rust"
level = "warn"

[[rules]]
name = "multi_language"
pointer = "/derived/distribution/language_count"
op = "gte"
value = 3
level = "warn"

[[rules]]
name = "has_ci"
pointer = "/metadata/ci_provider"
op = "exists"
level = "error"
"#;

    let policy = PolicyConfig::from_toml(toml).unwrap();
    let receipt = sample_receipt();
    let result = evaluate_policy(&receipt, &policy);

    assert!(result.passed);
    assert_eq!(result.rule_results.len(), 4);
}

// ============================================================================
// Workflow: fail_fast stops at first error
// ============================================================================

#[test]
fn workflow_fail_fast_stops_at_first_error_rule() {
    let receipt = json!({
        "tokens": 1_000_000,
        "code": 50,
        "license": "GPL"
    });

    let policy = PolicyConfig {
        rules: vec![
            PolicyRule {
                name: "tokens_ok".into(),
                pointer: "/tokens".into(),
                op: RuleOperator::Lte,
                value: Some(json!(500_000)),
                values: None,
                negate: false,
                level: RuleLevel::Error,
                message: None,
            },
            PolicyRule {
                name: "code_ok".into(),
                pointer: "/code".into(),
                op: RuleOperator::Gte,
                value: Some(json!(100)),
                values: None,
                negate: false,
                level: RuleLevel::Error,
                message: None,
            },
        ],
        fail_fast: true,
        allow_missing: false,
    };

    let result = evaluate_policy(&receipt, &policy);
    assert!(!result.passed);
    // First rule fails, so second rule is never evaluated
    assert_eq!(result.rule_results.len(), 1);
    assert_eq!(result.rule_results[0].name, "tokens_ok");
}

// ============================================================================
// Workflow: allow_missing for gradual adoption
// ============================================================================

#[test]
fn workflow_allow_missing_for_partial_receipts() {
    // Simulate a receipt from an older version that lacks some fields
    let partial_receipt = json!({
        "derived": {
            "totals": {"code": 5000}
        }
    });

    let policy = PolicyConfig {
        rules: vec![
            PolicyRule {
                name: "min_code".into(),
                pointer: "/derived/totals/code".into(),
                op: RuleOperator::Gte,
                value: Some(json!(100)),
                values: None,
                negate: false,
                level: RuleLevel::Error,
                message: None,
            },
            PolicyRule {
                name: "max_complexity".into(),
                pointer: "/complexity/avg".into(),
                op: RuleOperator::Lte,
                value: Some(json!(10.0)),
                values: None,
                negate: false,
                level: RuleLevel::Error,
                message: None,
            },
        ],
        fail_fast: false,
        allow_missing: true,
    };

    let result = evaluate_policy(&partial_receipt, &policy);
    assert!(result.passed); // Missing complexity is allowed
    assert_eq!(result.errors, 0);
}

// ============================================================================
// Workflow: Ratchet-based quality gate
// ============================================================================

#[test]
fn workflow_ratchet_all_metrics_within_bounds() {
    let baseline = baseline_receipt();
    let current = current_receipt();

    let config = RatchetConfig {
        rules: vec![
            RatchetRule {
                pointer: "/complexity/avg_cyclomatic".into(),
                max_increase_pct: Some(15.0), // Allow up to 15% increase
                max_value: Some(10.0),         // Hard ceiling
                level: RuleLevel::Error,
                description: Some("Cyclomatic complexity".into()),
            },
            RatchetRule {
                pointer: "/complexity/max_cyclomatic".into(),
                max_increase_pct: None,
                max_value: Some(25.0), // Hard ceiling only
                level: RuleLevel::Error,
                description: Some("Max cyclomatic complexity".into()),
            },
            RatchetRule {
                pointer: "/derived/totals/tokens".into(),
                max_increase_pct: Some(20.0),
                max_value: Some(500_000.0),
                level: RuleLevel::Warn,
                description: Some("Token budget".into()),
            },
        ],
        fail_fast: false,
        allow_missing_baseline: false,
        allow_missing_current: false,
    };

    let result = evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(result.passed);
    assert_eq!(result.ratchet_results.len(), 3);
    assert_eq!(result.errors, 0);

    // Verify detailed results
    let complexity = &result.ratchet_results[0];
    assert!(complexity.passed);
    assert_eq!(complexity.baseline_value, Some(4.2));
    assert_eq!(complexity.current_value, 4.5);
    assert!(complexity.change_pct.unwrap() > 0.0); // Increased
}

#[test]
fn workflow_ratchet_regression_detected() {
    let baseline = json!({
        "complexity": {"avg_cyclomatic": 4.0}
    });
    let current = json!({
        "complexity": {"avg_cyclomatic": 8.0}
    }); // 100% increase

    let config = RatchetConfig {
        rules: vec![RatchetRule {
            pointer: "/complexity/avg_cyclomatic".into(),
            max_increase_pct: Some(10.0),
            max_value: None,
            level: RuleLevel::Error,
            description: Some("Complexity regression".into()),
        }],
        fail_fast: false,
        allow_missing_baseline: false,
        allow_missing_current: false,
    };

    let result = evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(!result.passed);
    assert_eq!(result.errors, 1);
    let rr = &result.ratchet_results[0];
    assert!(!rr.passed);
    assert!(rr.message.contains("100.00%"));
    assert!(rr.message.contains("exceeds"));
}

#[test]
fn workflow_ratchet_with_missing_baseline_allowed() {
    // First time running — no baseline yet
    let baseline = json!({});
    let current = json!({"complexity": {"avg": 5.0}});

    let config = RatchetConfig {
        rules: vec![RatchetRule {
            pointer: "/complexity/avg".into(),
            max_increase_pct: Some(10.0),
            max_value: Some(15.0),
            level: RuleLevel::Error,
            description: None,
        }],
        fail_fast: false,
        allow_missing_baseline: true,
        allow_missing_current: false,
    };

    let result = evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(result.passed);
}

#[test]
fn workflow_ratchet_fail_fast_stops_on_first_error() {
    let baseline = json!({"a": 10.0, "b": 10.0});
    let current = json!({"a": 30.0, "b": 30.0}); // Both 200% increase

    let config = RatchetConfig {
        rules: vec![
            RatchetRule {
                pointer: "/a".into(),
                max_increase_pct: Some(10.0),
                max_value: None,
                level: RuleLevel::Error,
                description: None,
            },
            RatchetRule {
                pointer: "/b".into(),
                max_increase_pct: Some(10.0),
                max_value: None,
                level: RuleLevel::Error,
                description: None,
            },
        ],
        fail_fast: true,
        allow_missing_baseline: false,
        allow_missing_current: false,
    };

    let result = evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(!result.passed);
    assert_eq!(result.ratchet_results.len(), 1); // Stopped after first
}

// ============================================================================
// Workflow: Combined policy + ratchet gate
// ============================================================================

#[test]
fn workflow_combined_policy_and_ratchet() {
    let receipt = sample_receipt();
    let baseline = baseline_receipt();
    let current = current_receipt();

    // Policy gate
    let policy = PolicyConfig {
        rules: vec![
            PolicyRule {
                name: "max_tokens".into(),
                pointer: "/derived/totals/tokens".into(),
                op: RuleOperator::Lte,
                value: Some(json!(500_000)),
                values: None,
                negate: false,
                level: RuleLevel::Error,
                message: None,
            },
            PolicyRule {
                name: "has_license".into(),
                pointer: "/metadata/license".into(),
                op: RuleOperator::Exists,
                value: None,
                values: None,
                negate: false,
                level: RuleLevel::Error,
                message: None,
            },
        ],
        fail_fast: false,
        allow_missing: false,
    };

    let policy_result = evaluate_policy(&receipt, &policy);
    assert!(policy_result.passed);

    // Ratchet gate
    let ratchet_config = RatchetConfig {
        rules: vec![RatchetRule {
            pointer: "/derived/totals/tokens".into(),
            max_increase_pct: Some(20.0),
            max_value: Some(500_000.0),
            level: RuleLevel::Error,
            description: Some("Token budget ratchet".into()),
        }],
        fail_fast: false,
        allow_missing_baseline: false,
        allow_missing_current: false,
    };

    let ratchet_result = evaluate_ratchet_policy(&ratchet_config, &baseline, &current);
    assert!(ratchet_result.passed);

    // Combined: both must pass
    let overall_passed = policy_result.passed && ratchet_result.passed;
    assert!(overall_passed);
}

// ============================================================================
// Workflow: TOML ratchet config
// ============================================================================

#[test]
fn workflow_toml_ratchet_config_roundtrip() {
    let toml = r#"
fail_fast = true
allow_missing_baseline = false
allow_missing_current = false

[[rules]]
pointer = "/complexity/avg_cyclomatic"
max_increase_pct = 10.0
max_value = 8.0
level = "error"
description = "Average cyclomatic complexity"

[[rules]]
pointer = "/derived/totals/tokens"
max_value = 500000.0
level = "warn"
description = "Token budget warning"
"#;

    let config = RatchetConfig::from_toml(toml).unwrap();
    assert!(config.fail_fast);
    assert!(!config.allow_missing_baseline);
    assert_eq!(config.rules.len(), 2);

    // Use the parsed config
    let baseline = json!({"complexity": {"avg_cyclomatic": 5.0}, "derived": {"totals": {"tokens": 200000}}});
    let current = json!({"complexity": {"avg_cyclomatic": 5.2}, "derived": {"totals": {"tokens": 250000}}});

    let result = evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(result.passed);
}

// ============================================================================
// Workflow: Edge cases in real scenarios
// ============================================================================

#[test]
fn workflow_empty_policy_always_passes() {
    let receipt = json!({"anything": "goes"});
    let policy = PolicyConfig::default();
    let result = evaluate_policy(&receipt, &policy);
    assert!(result.passed);
    assert_eq!(result.rule_results.len(), 0);
    assert_eq!(result.errors, 0);
    assert_eq!(result.warnings, 0);
}

#[test]
fn workflow_empty_ratchet_config_always_passes() {
    let baseline = json!({});
    let current = json!({});
    let config = RatchetConfig::default();
    let result = evaluate_ratchet_policy(&config, &baseline, &current);
    assert!(result.passed);
    assert_eq!(result.ratchet_results.len(), 0);
}

#[test]
fn workflow_large_number_of_rules() {
    let receipt = json!({
        "metrics": (0..50).map(|i| (format!("m{}", i), json!(i))).collect::<serde_json::Map<String, serde_json::Value>>()
    });

    // Create 50 rules, all checking their respective metrics
    let rules: Vec<PolicyRule> = (0..50)
        .map(|i| PolicyRule {
            name: format!("check_m{}", i),
            pointer: format!("/metrics/m{}", i),
            op: RuleOperator::Gte,
            value: Some(json!(0)),
            values: None,
            negate: false,
            level: RuleLevel::Error,
            message: None,
        })
        .collect();

    let policy = PolicyConfig {
        rules,
        fail_fast: false,
        allow_missing: false,
    };

    let result = evaluate_policy(&receipt, &policy);
    assert!(result.passed);
    assert_eq!(result.rule_results.len(), 50);
}

#[test]
fn workflow_gate_result_serialization() {
    let receipt = json!({"tokens": 100, "code": 500});
    let policy = PolicyConfig {
        rules: vec![
            PolicyRule {
                name: "token_check".into(),
                pointer: "/tokens".into(),
                op: RuleOperator::Lte,
                value: Some(json!(1000)),
                values: None,
                negate: false,
                level: RuleLevel::Error,
                message: None,
            },
            PolicyRule {
                name: "code_check".into(),
                pointer: "/code".into(),
                op: RuleOperator::Gte,
                value: Some(json!(100)),
                values: None,
                negate: false,
                level: RuleLevel::Warn,
                message: None,
            },
        ],
        fail_fast: false,
        allow_missing: false,
    };

    let result = evaluate_policy(&receipt, &policy);

    // Verify GateResult can be serialized to JSON
    let json_str = serde_json::to_string(&result).expect("GateResult should serialize");
    let parsed: serde_json::Value =
        serde_json::from_str(&json_str).expect("Should parse back as JSON");

    assert_eq!(parsed["passed"], json!(true));
    assert_eq!(parsed["errors"], json!(0));
    assert_eq!(parsed["warnings"], json!(0));
    assert!(parsed["rule_results"].is_array());
    assert_eq!(parsed["rule_results"].as_array().unwrap().len(), 2);
}

#[test]
fn workflow_ratchet_result_serialization() {
    let baseline = json!({"complexity": 5.0});
    let current = json!({"complexity": 5.5});

    let config = RatchetConfig {
        rules: vec![RatchetRule {
            pointer: "/complexity".into(),
            max_increase_pct: Some(20.0),
            max_value: None,
            level: RuleLevel::Error,
            description: Some("Complexity check".into()),
        }],
        fail_fast: false,
        allow_missing_baseline: false,
        allow_missing_current: false,
    };

    let result = evaluate_ratchet_policy(&config, &baseline, &current);

    let json_str = serde_json::to_string(&result).expect("RatchetGateResult should serialize");
    let parsed: serde_json::Value =
        serde_json::from_str(&json_str).expect("Should parse back as JSON");

    assert_eq!(parsed["passed"], json!(true));
    assert!(parsed["ratchet_results"].is_array());
}

// ============================================================================
// Workflow: Realistic PR merge gate
// ============================================================================

#[test]
fn workflow_pr_merge_gate_scenario() {
    // Simulate what a CI pipeline would do:
    // 1. Run tokmd analyze on the PR branch
    // 2. Evaluate policy rules
    // 3. Compare against baseline with ratchet rules

    let pr_receipt = json!({
        "derived": {
            "totals": {
                "files": 155,
                "code": 12500,
                "comments": 2100,
                "blanks": 1550,
                "tokens": 290000
            }
        },
        "metadata": {
            "license": "MIT",
            "has_readme": true,
            "has_tests": true
        },
        "complexity": {
            "avg_cyclomatic": 4.3,
            "max_cyclomatic": 19
        }
    });

    // Step 1: Policy gate
    let policy_toml = r#"
fail_fast = false
allow_missing = false

[[rules]]
name = "token_budget"
pointer = "/derived/totals/tokens"
op = "lte"
value = 500000
level = "error"
message = "PR exceeds token budget"

[[rules]]
name = "has_tests"
pointer = "/metadata/has_tests"
op = "eq"
value = true
level = "error"
message = "Tests are required"

[[rules]]
name = "license_approved"
pointer = "/metadata/license"
op = "in"
values = ["MIT", "Apache-2.0"]
level = "error"

[[rules]]
name = "readme_present"
pointer = "/metadata/has_readme"
op = "exists"
level = "warn"
"#;
    let policy = PolicyConfig::from_toml(policy_toml).unwrap();
    let policy_result = evaluate_policy(&pr_receipt, &policy);
    assert!(policy_result.passed, "Policy gate should pass");

    // Step 2: Ratchet gate against main branch baseline
    let main_baseline = json!({
        "derived": {"totals": {"tokens": 280000}},
        "complexity": {"avg_cyclomatic": 4.2, "max_cyclomatic": 18}
    });

    let ratchet_toml = r#"
fail_fast = false
allow_missing_baseline = false
allow_missing_current = false

[[rules]]
pointer = "/complexity/avg_cyclomatic"
max_increase_pct = 10.0
max_value = 8.0
level = "error"
description = "Average complexity ratchet"

[[rules]]
pointer = "/complexity/max_cyclomatic"
max_value = 25.0
level = "error"
description = "Max complexity ceiling"

[[rules]]
pointer = "/derived/totals/tokens"
max_increase_pct = 15.0
max_value = 500000.0
level = "warn"
description = "Token growth monitor"
"#;
    let ratchet_config = RatchetConfig::from_toml(ratchet_toml).unwrap();
    let ratchet_result = evaluate_ratchet_policy(&ratchet_config, &main_baseline, &pr_receipt);
    assert!(ratchet_result.passed, "Ratchet gate should pass");

    // Combined decision
    let merge_allowed = policy_result.passed && ratchet_result.passed;
    assert!(merge_allowed, "PR should be allowed to merge");
}
