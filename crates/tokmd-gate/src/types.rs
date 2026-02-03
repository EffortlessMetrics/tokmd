//! Policy and rule type definitions.

use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

/// Errors from policy evaluation.
#[derive(Debug, Error)]
pub enum GateError {
    #[error("Failed to read policy file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to parse policy TOML: {0}")]
    TomlError(#[from] toml::de::Error),

    #[error("Invalid JSON pointer: {0}")]
    InvalidPointer(String),

    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },

    #[error("Invalid operator '{op}' for type '{value_type}'")]
    InvalidOperator { op: String, value_type: String },

    #[error("Rule '{name}' missing required field: {field}")]
    MissingField { name: String, field: String },
}

/// Root policy configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PolicyConfig {
    /// Policy rules to evaluate.
    pub rules: Vec<PolicyRule>,

    /// Stop evaluation on first error.
    #[serde(default)]
    pub fail_fast: bool,

    /// Allow missing values (treat as pass) instead of error.
    #[serde(default)]
    pub allow_missing: bool,
}

impl PolicyConfig {
    /// Parse policy from TOML string.
    pub fn from_toml(s: &str) -> Result<Self, GateError> {
        Ok(toml::from_str(s)?)
    }

    /// Load policy from a TOML file.
    pub fn from_file(path: &Path) -> Result<Self, GateError> {
        let content = std::fs::read_to_string(path)?;
        Self::from_toml(&content)
    }
}

/// A single policy rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    /// Human-readable name for the rule.
    pub name: String,

    /// JSON Pointer to the value to check (RFC 6901).
    pub pointer: String,

    /// Comparison operator.
    pub op: RuleOperator,

    /// Single value for comparison (for >, <, ==, etc.).
    #[serde(default)]
    pub value: Option<serde_json::Value>,

    /// Multiple values for "in" operator.
    #[serde(default)]
    pub values: Option<Vec<serde_json::Value>>,

    /// Negate the result (NOT).
    #[serde(default)]
    pub negate: bool,

    /// Rule severity level.
    #[serde(default)]
    pub level: RuleLevel,

    /// Custom failure message.
    #[serde(default)]
    pub message: Option<String>,
}

/// Comparison operators for rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RuleOperator {
    /// Greater than (>)
    Gt,
    /// Greater than or equal (>=)
    Gte,
    /// Less than (<)
    Lt,
    /// Less than or equal (<=)
    Lte,
    /// Equal (==)
    #[default]
    Eq,
    /// Not equal (!=)
    Ne,
    /// Value is in list
    In,
    /// String/array contains value
    Contains,
    /// JSON pointer exists (value is present)
    Exists,
}

impl std::fmt::Display for RuleOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuleOperator::Gt => write!(f, ">"),
            RuleOperator::Gte => write!(f, ">="),
            RuleOperator::Lt => write!(f, "<"),
            RuleOperator::Lte => write!(f, "<="),
            RuleOperator::Eq => write!(f, "=="),
            RuleOperator::Ne => write!(f, "!="),
            RuleOperator::In => write!(f, "in"),
            RuleOperator::Contains => write!(f, "contains"),
            RuleOperator::Exists => write!(f, "exists"),
        }
    }
}

/// Rule severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum RuleLevel {
    /// Warning - does not fail the gate.
    Warn,
    /// Error - fails the gate.
    #[default]
    Error,
}

/// Result of evaluating the entire policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateResult {
    /// Overall pass/fail.
    pub passed: bool,

    /// Individual rule results.
    pub rule_results: Vec<RuleResult>,

    /// Count of errors.
    pub errors: usize,

    /// Count of warnings.
    pub warnings: usize,
}

impl GateResult {
    /// Create a new gate result from rule results.
    pub fn from_results(rule_results: Vec<RuleResult>) -> Self {
        let errors = rule_results
            .iter()
            .filter(|r| !r.passed && r.level == RuleLevel::Error)
            .count();
        let warnings = rule_results
            .iter()
            .filter(|r| !r.passed && r.level == RuleLevel::Warn)
            .count();
        let passed = errors == 0;

        Self {
            passed,
            rule_results,
            errors,
            warnings,
        }
    }
}

/// Result of evaluating a single rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleResult {
    /// Rule name.
    pub name: String,

    /// Whether the rule passed.
    pub passed: bool,

    /// Rule level (error/warn).
    pub level: RuleLevel,

    /// Actual value found (if any).
    pub actual: Option<serde_json::Value>,

    /// Expected value or condition.
    pub expected: String,

    /// Failure message.
    pub message: Option<String>,
}

/// Ratchet rule for gradual improvement.
///
/// Ratchet rules enforce that metrics don't regress beyond acceptable bounds
/// when compared to a baseline. This enables gradual quality improvement by
/// allowing teams to "ratchet" down thresholds over time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatchetRule {
    /// JSON pointer to the metric (e.g., "/complexity/avg_cyclomatic").
    pub pointer: String,

    /// Maximum allowed increase percentage from baseline.
    /// For example, 10.0 means the current value can be at most 10% higher than baseline.
    #[serde(default)]
    pub max_increase_pct: Option<f64>,

    /// Maximum allowed absolute value.
    /// This acts as a hard ceiling regardless of baseline.
    #[serde(default)]
    pub max_value: Option<f64>,

    /// Rule severity level.
    #[serde(default)]
    pub level: RuleLevel,

    /// Human-readable description of the rule.
    #[serde(default)]
    pub description: Option<String>,
}

/// Result of ratchet evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatchetResult {
    /// The rule that was evaluated.
    pub rule: RatchetRule,

    /// Whether the ratchet check passed.
    pub passed: bool,

    /// Baseline value (if found).
    pub baseline_value: Option<f64>,

    /// Current value.
    pub current_value: f64,

    /// Percentage change from baseline (if baseline exists).
    pub change_pct: Option<f64>,

    /// Human-readable message describing the result.
    pub message: String,
}

/// Configuration for ratchet rules.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct RatchetConfig {
    /// Ratchet rules to evaluate.
    pub rules: Vec<RatchetRule>,

    /// Stop evaluation on first error.
    #[serde(default)]
    pub fail_fast: bool,

    /// Allow missing baseline values (treat as pass) instead of error.
    #[serde(default)]
    pub allow_missing_baseline: bool,

    /// Allow missing current values (treat as pass) instead of error.
    #[serde(default)]
    pub allow_missing_current: bool,
}

impl RatchetConfig {
    /// Parse ratchet config from TOML string.
    pub fn from_toml(s: &str) -> Result<Self, GateError> {
        Ok(toml::from_str(s)?)
    }

    /// Load ratchet config from a TOML file.
    pub fn from_file(path: &Path) -> Result<Self, GateError> {
        let content = std::fs::read_to_string(path)?;
        Self::from_toml(&content)
    }
}

/// Overall result of ratchet evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatchetGateResult {
    /// Overall pass/fail.
    pub passed: bool,

    /// Individual ratchet results.
    pub ratchet_results: Vec<RatchetResult>,

    /// Count of errors.
    pub errors: usize,

    /// Count of warnings.
    pub warnings: usize,
}

impl RatchetGateResult {
    /// Create a new ratchet gate result from ratchet results.
    pub fn from_results(ratchet_results: Vec<RatchetResult>) -> Self {
        let errors = ratchet_results
            .iter()
            .filter(|r| !r.passed && r.rule.level == RuleLevel::Error)
            .count();
        let warnings = ratchet_results
            .iter()
            .filter(|r| !r.passed && r.rule.level == RuleLevel::Warn)
            .count();
        let passed = errors == 0;

        Self {
            passed,
            ratchet_results,
            errors,
            warnings,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_policy() {
        let toml = r#"
fail_fast = true
allow_missing = false

[[rules]]
name = "max_tokens"
pointer = "/derived/totals/tokens"
op = "lte"
value = 500000
level = "error"
message = "Too many tokens"

[[rules]]
name = "has_license"
pointer = "/license/effective"
op = "exists"
level = "warn"
"#;
        let policy = PolicyConfig::from_toml(toml).unwrap();
        assert!(policy.fail_fast);
        assert!(!policy.allow_missing);
        assert_eq!(policy.rules.len(), 2);
        assert_eq!(policy.rules[0].name, "max_tokens");
        assert_eq!(policy.rules[0].op, RuleOperator::Lte);
        assert_eq!(policy.rules[1].op, RuleOperator::Exists);
    }

    #[test]
    fn test_gate_result() {
        let results = vec![
            RuleResult {
                name: "rule1".into(),
                passed: true,
                level: RuleLevel::Error,
                actual: None,
                expected: "test".into(),
                message: None,
            },
            RuleResult {
                name: "rule2".into(),
                passed: false,
                level: RuleLevel::Warn,
                actual: None,
                expected: "test".into(),
                message: Some("Warning".into()),
            },
        ];

        let gate = GateResult::from_results(results);
        assert!(gate.passed); // Only warns, no errors
        assert_eq!(gate.errors, 0);
        assert_eq!(gate.warnings, 1);
    }

    #[test]
    fn test_policy_from_file() {
        // Kills mutant: PolicyConfig::from_file -> Ok(Default::default()).
        use std::time::{SystemTime, UNIX_EPOCH};

        let toml = r#"
fail_fast = true
allow_missing = false

[[rules]]
name = "max_tokens"
pointer = "/derived/totals/tokens"
op = "lte"
value = 500000
level = "error"
"#;

        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("tokmd-gate-policy-{nanos}.toml"));
        std::fs::write(&path, toml).unwrap();

        let policy = PolicyConfig::from_file(&path).unwrap();
        let _ = std::fs::remove_file(&path);

        assert!(policy.fail_fast);
        assert_eq!(policy.rules.len(), 1);
        assert_eq!(policy.rules[0].name, "max_tokens");
        assert_eq!(policy.rules[0].op, RuleOperator::Lte);
    }

    #[test]
    fn test_rule_operator_display() {
        // Kills mutant in Display impl.
        assert_eq!(RuleOperator::Gt.to_string(), ">");
        assert_eq!(RuleOperator::Gte.to_string(), ">=");
        assert_eq!(RuleOperator::Lt.to_string(), "<");
        assert_eq!(RuleOperator::Lte.to_string(), "<=");
        assert_eq!(RuleOperator::Eq.to_string(), "==");
        assert_eq!(RuleOperator::Ne.to_string(), "!=");
        assert_eq!(RuleOperator::In.to_string(), "in");
        assert_eq!(RuleOperator::Contains.to_string(), "contains");
        assert_eq!(RuleOperator::Exists.to_string(), "exists");
    }

    #[test]
    fn test_gate_result_counts_only_failed_rules() {
        // Kills `&&` -> `||` mutant in warning counting by including a passed WARN.
        let results = vec![
            RuleResult {
                name: "passed_warn".into(),
                passed: true,
                level: RuleLevel::Warn,
                actual: None,
                expected: "x".into(),
                message: None,
            },
            RuleResult {
                name: "failed_warn".into(),
                passed: false,
                level: RuleLevel::Warn,
                actual: None,
                expected: "x".into(),
                message: Some("warn".into()),
            },
        ];

        let gate = GateResult::from_results(results);
        assert!(gate.passed); // warns only
        assert_eq!(gate.errors, 0);
        assert_eq!(gate.warnings, 1);
    }
}
