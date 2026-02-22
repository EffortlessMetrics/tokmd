# tokmd-analysis-derived

Derived metrics builder for `tokmd-analysis`.

It computes deterministic aggregate analytics from `ExportData`, including:

- totals and rates
- doc/test/whitespace ratios
- file distributions and histogram outputs
- complexity proxy signals like top files, gini/polynomial style indicators
- optional context-window fit and tree rendering

This is consumed by `tokmd-analysis` and can be used by other adapters
without pulling optional analysis feature dependencies.