# tokmd-analysis-maintainability

This microcrate owns maintainability-index scoring rules and Halstead-based
maintainability updates for `tokmd` analysis receipts.

## Purpose

- Keep SEI maintainability math in one place.
- Keep letter-grade thresholds stable (`A` / `B` / `C`).
- Keep Halstead-to-maintainability integration behavior deterministic.

## API

- `compute_maintainability_index` - compute simplified or full SEI MI.
- `attach_halstead_metrics` - attach Halstead metrics and refresh MI when valid.
