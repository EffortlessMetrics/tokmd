# tokmd-analysis-maintainability

Maintainability scoring helpers for tokmd analysis receipts.

## Problem

You want maintainability metrics derived from complexity and size without
re-implementing the scoring math in the caller.

## What it gives you

- `compute_maintainability_index`
- `attach_halstead_metrics`

## Integration notes

- Uses the simplified or full SEI-style formula depending on Halstead volume.
- Recomputes the index only when the existing complexity report already has a
  maintainability slot and Halstead volume is positive.
- Keeps the Halstead merge logic local to the analysis layer.

## Go deeper

### Reference

- [Source](src/lib.rs)
- [tokmd-analysis-halstead](../tokmd-analysis-halstead/README.md)

### Explanation

- [Architecture](../../docs/architecture.md)
- [Philosophy](../../docs/explanation.md)
