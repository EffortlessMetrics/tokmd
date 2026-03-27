# tokmd-analysis-fun

Novelty enrichments for tokmd analysis receipts.

## Problem

You want optional novelty signals, such as eco-labels, without tying them to
the core analysis orchestrator.

## What it gives you

- `build_fun_report`
- `EcoLabel` output on the analysis receipt

## Integration notes

- Consumes a `DerivedReport` and emits `FunReport`.
- Currently centers on the size-based eco-label path used by `AnalysisPreset::Fun`.
- Keeps novelty output isolated so it can evolve independently.

## Go deeper

### Reference

- [Source](src/lib.rs)
- [Analysis types](../tokmd-analysis-types/README.md)

### Explanation

- [Architecture](../../docs/architecture.md)
- [Philosophy](../../docs/explanation.md)
