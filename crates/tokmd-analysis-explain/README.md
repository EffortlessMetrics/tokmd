# tokmd-analysis-explain

Metric and finding explanation catalog for tokmd analysis receipts.

## Problem

You need stable human-readable explanations for metric keys and findings
without pulling in the full analysis orchestrator.

## What it gives you

- `lookup`
- `catalog`

## Integration notes

- Normalizes metric keys and supports alias lookup.
- Returns a deterministic catalog order for stable docs and tests.
- Keeps the explanation surface separate from receipt generation.

## Go deeper

### Reference

- [Source](src/lib.rs)
- [Schema](../../docs/SCHEMA.md)
- [Schema JSON](../../docs/schema.json)

### Explanation

- [Architecture](../../docs/architecture.md)
- [Design](../../docs/design.md)
- [Philosophy](../../docs/explanation.md)
