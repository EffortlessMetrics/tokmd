# tokmd-analysis-complexity

Complexity scoring and histogram generation for analysis receipts.

## Problem
You need per-file complexity and technical-debt signals without folding parsing logic into the receipt model.

## What it gives you
- `build_complexity_report`
- `generate_complexity_histogram`
- `ComplexityReport`, `FileComplexity`, `ComplexityHistogram`

## Integration notes
- No feature flags.
- Uses `AnalysisLimits` to cap bytes and file counts.
- Can attach per-function details when requested; relies on `tokmd-content` and maintainability helpers.

## Go deeper
- Tutorial: [Tutorial](../../docs/tutorial.md)
- How-to: [Recipes](../../docs/recipes.md)
- Reference: [Architecture](../../docs/architecture.md), [Root README](../../README.md)
- Explanation: [Explanation](../../docs/explanation.md)
