# tokmd-analysis-topics

Topic-cloud enrichment for analysis receipts.

## Problem

You want path-based topic signals without coupling the score-building logic to
the main analysis orchestrator.

## What it gives you

- `build_topic_clouds`

## Integration notes

- Builds per-module and overall topic clouds from `ExportData`.
- Uses token weights, stopwords, and deterministic TF-IDF-style scoring.
- Keeps topic enrichment isolated so preset orchestration stays small.

## Go deeper

### Reference

- [Source](src/lib.rs)

### Explanation

- [Architecture](../../docs/architecture.md)
- [Philosophy](../../docs/explanation.md)
