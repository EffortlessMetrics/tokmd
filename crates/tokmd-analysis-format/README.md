# tokmd-analysis-format

Formatting and rendering for analysis receipts.

## Problem
You need receipt output in several formats without mixing rendering logic into the data model.

## What it gives you
- `render`
- `RenderedOutput`
- Markdown, JSON, JSON-LD, XML, SVG, Mermaid, OBJ, MIDI, tree, and HTML output paths

## Integration notes
- HTML rendering is delegated to `tokmd-analysis-html`.
- The optional `fun` feature enables novelty output via `tokmd-fun`.
- Rendering preserves whatever the receipt already contains.

## Go deeper
- Tutorial: [Tutorial](../../docs/tutorial.md)
- How-to: [Recipes](../../docs/recipes.md)
- Reference: [Architecture](../../docs/architecture.md), [Reference CLI](../../docs/reference-cli.md)
- Explanation: [Explanation](../../docs/explanation.md)
