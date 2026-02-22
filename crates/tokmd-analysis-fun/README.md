# tokmd-analysis-fun

Microcrate for novelty/enrichment computations used by `tokmd-analysis`.

## Current Responsibility

- Compute the `FunReport` eco-label in `build_fun_report`.

## API

```rust
pub fn build_fun_report(derived: &DerivedReport) -> FunReport;
```

## Notes

- Keeps novelty logic in a separate crate behind the `fun` feature in `tokmd-analysis`.
- This crate has no formatting/rendering dependencies.
