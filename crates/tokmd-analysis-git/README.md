# tokmd-analysis-git

Git-history enrichments for tokmd analysis presets.

## Problem

You need churn, freshness, and coupling signals from git history without
mixing them into the core scan or format layers.

## What it gives you

- `build_git_report`
- `build_predictive_churn_report`

## Integration notes

- Consumes git-derived inputs and produces analysis-side enrichment reports.
- Keeps history-based heuristics separate from the receipt model and renderer.
- Deterministic ordering is preserved through the shared analysis types.

## Go deeper

### Reference

- [Source](src/lib.rs)
- [Churn implementation](src/churn.rs)

### Explanation

- [Architecture](../../docs/architecture.md)
- [Philosophy](../../docs/explanation.md)
