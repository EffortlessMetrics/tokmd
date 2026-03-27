# tokmd-analysis-halstead

Halstead metric helpers for tokmd analysis receipts.

## Problem

You need Halstead metrics from source text without pulling the full analysis
pipeline into your caller.

## What it gives you

- `is_halstead_lang`
- `operators_for_lang`
- `FileTokenCounts`
- `tokenize_for_halstead`
- `build_halstead_report`

## Integration notes

- Supports a curated language set only.
- Reads text through `tokmd-content` and caps file sizes through analysis
  limits.
- Produces deterministic aggregate metrics for the shared analysis receipts.

## Go deeper

### Reference

- [Source](src/lib.rs)
- [tokmd-content](../tokmd-content/README.md)

### Explanation

- [Architecture](../../docs/architecture.md)
- [Philosophy](../../docs/explanation.md)
