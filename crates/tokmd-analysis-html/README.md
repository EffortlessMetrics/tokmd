# tokmd-analysis-html

Single-responsibility microcrate for rendering `AnalysisReceipt` values as a self-contained HTML report.

## What it does

- Produces a static HTML report from analysis receipts.
- Renders metric cards, top-file table rows, and embedded treemap JSON payload.
- Applies HTML and JSON escaping for browser-safe output.

## API

- `render(receipt: &AnalysisReceipt) -> String`
