# tokmd-analysis-effort

Effort estimation scaffolding and outputs for analysis receipts.

## Problem
You need an effort model that stays receipt-driven instead of pulling in a separate estimation service.

## What it gives you
- `build_effort_report`
- `build_size_basis`, `build_drivers`, `build_delta`
- `apply_monte_carlo`, `apply_uncertainty`
- `EffortRequest`, `EffortModelKind`, `EffortLayer`, `DeltaInput`

## Integration notes
- No default features.
- Optional `git` support adds commit-history context for effort layers and delta estimates.
- The output contract lives in `tokmd-analysis-types` as `EffortEstimateReport`.

## Go deeper
- Tutorial: [Tutorial](../../docs/tutorial.md)
- How-to: [Recipes](../../docs/recipes.md)
- Reference: [Architecture](../../docs/architecture.md), [Root README](../../README.md)
- Explanation: [Explanation](../../docs/explanation.md)
