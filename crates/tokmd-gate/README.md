# tokmd-gate

Evaluate JSON Pointer policies against tokmd receipts.

## Problem

CI needs explicit pass/fail rules over structured receipts, not ad hoc scripts.

## What it gives you

- `PolicyConfig`, `PolicyRule`, `RuleOperator`, `RuleLevel`
- `evaluate_policy` for receipt checks
- `evaluate_ratchet_policy` for trend gates
- `resolve_pointer` for JSON Pointer lookup

## Quick use / integration notes

```toml
[[rules]]
name = "max_tokens"
pointer = "/derived/totals/tokens"
op = "<="
value = 500000
level = "error"
```

Supported operators: `>`, `>=`, `<`, `<=`, `==`, `!=`, `in`, `contains`, and `exists`.

## Go deeper

### How-to

- `../../docs/reference-cli.md`

### Reference

- `src/lib.rs`

### Explanation

- `../../docs/explanation.md`
