# tokmd-analysis-util

A tiny microcrate for shared, deterministic analysis helpers used across analysis enrichers.

## API

- `AnalysisLimits`
- `normalize_path`
- `normalize_root`
- path and language heuristics for derived/content metrics
- compatibility re-exports for numeric helpers from `tokmd-math`

This crate is intentionally small and stable so the higher-level analysis crates
can evolve independently.
