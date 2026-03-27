# tokmd-context-git

Rank files for context selection using git churn signals.

## Problem

Use this crate when you want a small, deterministic way to rank files by
hotspot and commit-count pressure for context or handoff bundles.

## What it gives you

- `GitScores` with `hotspots` and `commit_counts`
- `compute_git_scores` behind the `git` feature
- `None` fallback when git support is disabled or unavailable
- shared path normalization via `tokmd-path`

## Quick use / integration notes

```toml
[dependencies]
tokmd-context-git = { workspace = true }
```

Enable the `git` feature when you want churn-backed ranking; leave it off for
lightweight builds.

## Go deeper

Tutorial: [Root README](../../README.md)
How-to: [Recipes](../../docs/recipes.md)
Reference: [Source](src/lib.rs)
Explanation: [Architecture](../../docs/architecture.md)
