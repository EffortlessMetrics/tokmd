# tokmd-analysis-topics

Tiered microcrate for topic-cloud enrichment used by analysis receipts.

## What it does

- Tokenizes file paths into lightweight terms.
- Computes weighted TF-like per-module topic scores.
- Emits module-level and overall top-k topic lists.

## API

- `build_topic_clouds(export: &ExportData) -> TopicClouds`
