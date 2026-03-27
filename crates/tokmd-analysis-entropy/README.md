# tokmd-analysis-entropy

Entropy profiling for suspicious or packed-looking files.

## Problem
You need a lightweight entropy signal without turning every file into a full content scan.

## What it gives you
- `build_entropy_report`
- `EntropyReport`, `EntropyFinding`, `EntropyClass`

## Integration notes
- No feature flags.
- Samples file heads and tails with `AnalysisLimits`.
- Designed to surface high-entropy outliers for the `security` preset.

## Go deeper
- Tutorial: [Tutorial](../../docs/tutorial.md)
- How-to: [Recipes](../../docs/recipes.md)
- Reference: [Architecture](../../docs/architecture.md), [Root README](../../README.md)
- Explanation: [Explanation](../../docs/explanation.md)
