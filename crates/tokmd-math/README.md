# tokmd-math

Single-responsibility microcrate for deterministic numeric helpers used across tokmd analysis crates.

## API

- `round_f64(value, decimals)` - deterministic decimal rounding helper
- `safe_ratio(numer, denom)` - zero-safe ratio helper (4 decimal places)
- `percentile(sorted, pct)` - percentile lookup over sorted integer values
- `gini_coefficient(sorted)` - inequality coefficient for sorted integer values

## Guarantees

- deterministic for identical inputs
- no allocation for core operations
- no unsafe code
