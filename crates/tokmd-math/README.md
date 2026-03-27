# tokmd-math

Deterministic numeric helpers for tokmd analysis crates.

## Problem
Analysis metrics should round, divide, and rank the same way every time.

## What it gives you
- `round_f64(value, decimals)`
- `safe_ratio(numer, denom)`
- `percentile(sorted, pct)`
- `gini_coefficient(sorted)`

## API / usage notes
- Use these helpers for presentation and report math, not for ad hoc business rules.
- They are designed for stable outputs on identical inputs.
- See `src/lib.rs` for the exact rounding and percentile behavior.

## Go deeper
- Tutorial: [tokmd README](../../README.md)
- How-to: [Recipes](../../docs/recipes.md)
- Reference: [Architecture](../../docs/architecture.md)
- Explanation: [Design](../../docs/design.md)
