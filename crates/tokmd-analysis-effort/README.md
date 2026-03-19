# tokmd-analysis-effort

Deterministic effort-estimation support for `tokmd` analysis receipts.

This crate builds `tokmd_analysis_types::EffortEstimateReport` values from
repository inventory data plus optional analysis enrichers. The model is
receipt-driven and local-only: it uses repository files and already-computed
analysis reports, and it does not call external services.

The effort pipeline is intentionally layered:

- build an authored-vs-total size basis
- run a deterministic baseline model over authored KLOC
- widen or narrow the estimate using observed repository signals
- explain the result with drivers and confidence reasons
- optionally attach a base/head delta estimate

Primary consumers are `tokmd-analysis` and downstream renderers that need a
stable effort semantics layer.
