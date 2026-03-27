# tokmd-analysis-near-dup

Near-duplicate detection for tokmd analysis receipts.

## Problem

You need a deterministic way to identify duplicate or near-duplicate files
without embedding that logic in the orchestrator.

## What it gives you

- `NearDupLimits`
- `build_near_dup_report`

## Integration notes

- Uses Winnowing fingerprints with deterministic sorting and clustering.
- Supports `Global`, `Module`, and `Lang` scopes.
- Accepts threshold, truncation, and exclude-pattern controls for large repos.

## Go deeper

### Reference

- [Source](src/lib.rs)

### Explanation

- [Architecture](../../docs/architecture.md)
- [Philosophy](../../docs/explanation.md)
