# tokmd-analysis-fingerprint

Corporate fingerprint enrichment from git history.

## Problem
You need an org-signal summary from commit history without coupling to the full analysis orchestrator.

## What it gives you
- `build_corporate_fingerprint`
- `CorporateFingerprint`
- Domain-level commit concentration signals

## Integration notes
- No feature flags.
- Depends on `tokmd-git` commit metadata.
- Designed for the `identity` analysis preset.

## Go deeper
- Tutorial: [Tutorial](../../docs/tutorial.md)
- How-to: [Recipes](../../docs/recipes.md)
- Reference: [Architecture](../../docs/architecture.md), [Root README](../../README.md)
- Explanation: [Explanation](../../docs/explanation.md)
