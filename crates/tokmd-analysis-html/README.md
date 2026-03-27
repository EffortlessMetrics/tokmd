# tokmd-analysis-html

HTML rendering for tokmd analysis receipts.

## Problem

You need a self-contained HTML report without mixing rendering concerns into
the analysis data model.

## What it gives you

- `render`

## Integration notes

- Renders a complete `AnalysisReceipt` into a single HTML document.
- Escapes embedded HTML and JSON so the output is browser-safe.
- Uses the bundled template in `src/templates/report.html`.

## Go deeper

### Reference

- [Source](src/lib.rs)
- [Template](src/templates/report.html)

### How-to

- [Recipes](../../docs/recipes.md)

### Explanation

- [Architecture](../../docs/architecture.md)
- [Philosophy](../../docs/explanation.md)
