# tokmd-badge

Build compact SVG badges for tokmd metrics.

## Problem

Use this crate when you need a deterministic badge renderer without pulling in
the CLI or the rest of the formatting pipeline.

## What it gives you

- `badge_svg(label, value) -> String`
- XML escaping for badge text
- compact two-segment SVG output sized from the label and value text

## Quick use / integration notes

```toml
[dependencies]
tokmd-badge = { workspace = true }
```

Feed it a label and value, then embed the returned SVG in a README or docs
page.

## Go deeper

Tutorial: [Root README](../../README.md)
How-to: [Generated badges](../../README.md#generated-badges)
Reference: [Source](src/lib.rs)
Explanation: [Architecture](../../docs/architecture.md)
