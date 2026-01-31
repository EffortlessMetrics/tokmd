# tokmd-gate

Policy evaluation engine for tokmd analysis receipts. Enables CI gating based on code metrics.

## Usage

```rust
use tokmd_gate::{PolicyConfig, evaluate_policy};
use serde_json::Value;

let receipt: Value = serde_json::from_str(receipt_json)?;
let policy = PolicyConfig::from_file("policy.toml")?;
let result = evaluate_policy(&receipt, &policy)?;

if !result.passed {
    eprintln!("Policy check failed: {} errors, {} warnings", result.errors, result.warnings);
}
```

## Policy File Format

```toml
fail_fast = false

[[rules]]
name = "max_tokens"
pointer = "/derived/totals/tokens"
op = "<="
value = 500000
level = "error"
message = "Codebase exceeds token budget"

[[rules]]
name = "min_doc_density"
pointer = "/derived/doc_density/total/ratio"
op = ">="
value = 0.1
level = "warn"
message = "Documentation below 10%"
```

## Supported Operators

- `>`, `>=`, `<`, `<=`: Numeric comparisons
- `==`, `!=`: Equality checks
- `in`: Value is in a list
- `contains`: String/array contains value
- `exists`: JSON pointer exists

## License

MIT OR Apache-2.0
