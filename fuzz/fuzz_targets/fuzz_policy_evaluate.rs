//! Fuzz target for policy evaluation logic.
//!
//! Tests `evaluate_policy()` with arbitrary JSON receipts and policy rules
//! to find panics or unexpected behavior in rule evaluation.

#![no_main]
use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use serde_json::Value;
use tokmd_gate::{PolicyConfig, PolicyRule, RuleLevel, RuleOperator, evaluate_policy};

#[derive(Debug, Arbitrary)]
struct EvaluateInput {
    /// Raw bytes for JSON receipt (will be parsed)
    receipt_bytes: Vec<u8>,
    /// Policy configuration
    policy: FuzzPolicy,
}

#[derive(Debug, Arbitrary)]
struct FuzzPolicy {
    rules: Vec<FuzzRule>,
    fail_fast: bool,
    allow_missing: bool,
}

#[derive(Debug, Arbitrary)]
struct FuzzRule {
    name: String,
    pointer: String,
    op: FuzzOperator,
    /// Numeric value for comparisons
    num_value: Option<i64>,
    /// String value for comparisons
    str_value: Option<String>,
    /// Multiple values for "in" operator
    values: Option<Vec<String>>,
    negate: bool,
    level: FuzzLevel,
}

#[derive(Debug, Arbitrary)]
enum FuzzOperator {
    Gt,
    Gte,
    Lt,
    Lte,
    Eq,
    Ne,
    In,
    Contains,
    Exists,
}

#[derive(Debug, Arbitrary)]
enum FuzzLevel {
    Warn,
    Error,
}

impl From<FuzzOperator> for RuleOperator {
    fn from(op: FuzzOperator) -> Self {
        match op {
            FuzzOperator::Gt => RuleOperator::Gt,
            FuzzOperator::Gte => RuleOperator::Gte,
            FuzzOperator::Lt => RuleOperator::Lt,
            FuzzOperator::Lte => RuleOperator::Lte,
            FuzzOperator::Eq => RuleOperator::Eq,
            FuzzOperator::Ne => RuleOperator::Ne,
            FuzzOperator::In => RuleOperator::In,
            FuzzOperator::Contains => RuleOperator::Contains,
            FuzzOperator::Exists => RuleOperator::Exists,
        }
    }
}

impl From<FuzzLevel> for RuleLevel {
    fn from(level: FuzzLevel) -> Self {
        match level {
            FuzzLevel::Warn => RuleLevel::Warn,
            FuzzLevel::Error => RuleLevel::Error,
        }
    }
}

impl From<FuzzRule> for PolicyRule {
    fn from(rule: FuzzRule) -> Self {
        let value = rule
            .num_value
            .map(|n| serde_json::json!(n))
            .or_else(|| rule.str_value.clone().map(|s| serde_json::json!(s)));

        let values = rule
            .values
            .map(|v| v.into_iter().map(|s| serde_json::json!(s)).collect());

        PolicyRule {
            name: rule.name,
            pointer: rule.pointer,
            op: rule.op.into(),
            value,
            values,
            negate: rule.negate,
            level: rule.level.into(),
            message: None,
        }
    }
}

impl From<FuzzPolicy> for PolicyConfig {
    fn from(policy: FuzzPolicy) -> Self {
        PolicyConfig {
            rules: policy.rules.into_iter().map(Into::into).collect(),
            fail_fast: policy.fail_fast,
            allow_missing: policy.allow_missing,
        }
    }
}

fuzz_target!(|input: EvaluateInput| {
    // Try to parse JSON receipt from bytes
    let Ok(json_str) = std::str::from_utf8(&input.receipt_bytes) else {
        return;
    };
    let Ok(receipt) = serde_json::from_str::<Value>(json_str) else {
        return;
    };

    // Convert fuzz policy to real policy
    let policy: PolicyConfig = input.policy.into();

    // Evaluate - should never panic
    let _ = evaluate_policy(&receipt, &policy);
});
