# tokmd-analysis-assets

Asset inventory and lockfile summaries for analysis receipts.

## Problem
You need file-asset and dependency counts without teaching the orchestrator how to walk the tree.

## What it gives you
- `build_assets_report`
- `build_dependency_report`
- `AssetReport`, `DependencyReport`, `LockfileReport`

## Integration notes
- No feature flags.
- Reads file sizes from the walk root and lockfiles from detected package roots.
- Truncates the top asset rows for stable receipts.

## Go deeper
- Tutorial: [Tutorial](../../docs/tutorial.md)
- How-to: [Recipes](../../docs/recipes.md)
- Reference: [Architecture](../../docs/architecture.md), [Root README](../../README.md)
- Explanation: [Explanation](../../docs/explanation.md)
