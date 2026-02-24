# tokmd-analysis-content

Content-oriented analysis primitives for `tokmd-analysis`.

This crate provides shared analysis implementations for:

- TODO/TODO-like tag counting
- Duplicate file detection (exact hash match)
- Import graph extraction (delegates language parsing to `tokmd-analysis-imports`)

It is intentionally small and single-purpose so it can be composed by
`tokmd-analysis` and any other consumers that want these reports without
bringing in the full analysis orchestration layer.
