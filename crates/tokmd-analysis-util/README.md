# tokmd-analysis-util

Shared normalization, limits, and math helpers for analysis code.

## Problem
You need deterministic helpers without dragging in orchestration or receipt-specific logic.

## What it gives you
- `AnalysisLimits`
- `normalize_path`, `normalize_root`
- `path_depth`, `is_test_path`, `is_infra_lang`
- `now_ms`, `empty_file_row`
- `gini_coefficient`, `percentile`, `round_f64`, `safe_ratio`

## Integration notes
- No feature flags.
- Pure helper crate with deterministic path and ratio utilities.
- Re-exports math helpers from `tokmd-math`.

## Go deeper
- Tutorial: [Tutorial](../../docs/tutorial.md)
- How-to: [Recipes](../../docs/recipes.md)
- Reference: [Architecture](../../docs/architecture.md), [Root README](../../README.md)
- Explanation: [Explanation](../../docs/explanation.md)
