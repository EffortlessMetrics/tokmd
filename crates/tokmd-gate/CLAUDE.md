# tokmd-gate

## Purpose

Policy evaluation engine for CI gating. This is a **Tier 3** crate that evaluates JSON pointer-based rules against analysis receipts.

## Responsibility

- Policy rule parsing from TOML configuration
- JSON Pointer resolution (RFC 6901)
- Rule evaluation with comparison operators
- Gate result aggregation with error/warning counts
- **NOT** for generating receipts (see tokmd-analysis)
- **NOT** for CLI integration (see tokmd CLI)

## Public API

```rust
/// Evaluate all policy rules against a JSON receipt
pub fn evaluate_policy(receipt: &Value, policy: &PolicyConfig) -> GateResult

/// Resolve a JSON Pointer string to a value in a JSON document
pub fn resolve_pointer<'a>(root: &'a Value, pointer: &str) -> Option<&'a Value>

/// Root policy configuration
pub struct PolicyConfig {
    pub rules: Vec<PolicyRule>,
    pub fail_fast: bool,
    pub allow_missing: bool,
}

/// A single policy rule
pub struct PolicyRule {
    pub name: String,
    pub pointer: String,
    pub op: RuleOperator,
    pub value: Option<Value>,
    pub values: Option<Vec<Value>>,
    pub negate: bool,
    pub level: RuleLevel,
    pub message: Option<String>,
}

/// Comparison operators
pub enum RuleOperator { Gt, Gte, Lt, Lte, Eq, Ne, In, Contains, Exists }

/// Rule severity levels
pub enum RuleLevel { Warn, Error }

/// Overall gate evaluation result
pub struct GateResult {
    pub passed: bool,
    pub rule_results: Vec<RuleResult>,
    pub errors: usize,
    pub warnings: usize,
}
```

## Implementation Details

- Uses RFC 6901 JSON Pointer syntax for value resolution
- Numeric comparisons support both integers and floats (converts to f64)
- String values in JSON are coerced to numbers for numeric comparisons
- `fail_fast` mode stops evaluation on first error-level failure
- `allow_missing` treats missing pointers as pass instead of error
- `negate` flag inverts any comparison result

## Use Cases

- CI/CD quality gates based on code metrics
- Token budget enforcement for LLM contexts
- Documentation coverage requirements
- Complexity thresholds for code review

## Policy File Format

```toml
fail_fast = false
allow_missing = false

[[rules]]
name = "max_tokens"
pointer = "/derived/totals/tokens"
op = "lte"
value = 500000
level = "error"
message = "Codebase exceeds token budget"

[[rules]]
name = "has_license"
pointer = "/license/effective"
op = "exists"
level = "warn"
```

## Dependencies

- `serde` / `serde_json` (JSON handling)
- `toml` (policy file parsing)
- `thiserror` (error types)

## Testing

```bash
cargo test -p tokmd-gate
```

Tests cover:
- All comparison operators (>, >=, <, <=, ==, !=)
- `in` operator for list membership
- `contains` for string/array containment
- `exists` for pointer presence
- `negate` flag inversion
- `fail_fast` behavior
- `allow_missing` behavior
- Numeric type coercion
- Edge cases (epsilon boundaries, non-scalar equality)

## Do NOT

- Add receipt generation logic (belongs in tokmd-analysis)
- Add CLI argument parsing (belongs in tokmd CLI)
- Modify pointer resolution without updating tests
- Add side effects to evaluation (keep it pure)
