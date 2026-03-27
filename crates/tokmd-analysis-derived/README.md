# tokmd-analysis-derived

Derived metrics and tree output for analysis receipts.

## Problem
You need receipt-level rollups without pushing aggregation logic into the CLI or formatter.

## What it gives you
- `derive_report`
- `build_tree`
- `DerivedReport`, `DerivedTotals`, `ReadingTimeReport`

## Integration notes
- No feature flags.
- Consumes export data plus optional `window_tokens`.
- Produces totals, distributions, reading-time, COCOMO, TODO, and integrity data.

## Go deeper
- Tutorial: [Tutorial](../../docs/tutorial.md)
- How-to: [Recipes](../../docs/recipes.md)
- Reference: [Architecture](../../docs/architecture.md), [Root README](../../README.md)
- Explanation: [Explanation](../../docs/explanation.md)
