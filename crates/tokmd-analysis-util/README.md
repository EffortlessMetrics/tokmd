# tokmd-analysis-util

A tiny microcrate for shared, deterministic analysis helpers used across analysis enrichers.

## API

- `AnalysisLimits`
- `normalize_path`
- `normalize_root`
- path and language heuristics for derived/content metrics
- numerical helpers (`percentile`, `safe_ratio`, `round_f64`, etc.)

This crate is intentionally small and stable so the higher-level analysis crates
can evolve independently.
