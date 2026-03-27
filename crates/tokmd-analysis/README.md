# tokmd-analysis

Orchestrates analysis presets and enrichers for tokmd receipts.

## Problem
You need one analysis entrypoint that can assemble receipt enrichments without wiring every leaf crate yourself.

## What it gives you
- `analyze`
- `AnalysisContext`, `AnalysisRequest`, `AnalysisPreset`
- `ImportGranularity`
- Re-exports of `AnalysisLimits`, `NearDupScope`, and `normalize_root`

## Integration notes
- Default features: `fun`, `topics`, `archetype`, `effort`.
- Optional features: `git`, `walk`, `content`, `halstead`, `effort`, `fun`, `topics`, `archetype`.
- Use this crate when you want preset-driven orchestration; use the leaf crates directly when you only need one report.

## Go deeper
- Tutorial: [Tutorial](../../docs/tutorial.md)
- How-to: [Recipes](../../docs/recipes.md)
- Reference: [Architecture](../../docs/architecture.md), [CLI reference](../../docs/reference-cli.md)
- Explanation: [Explanation](../../docs/explanation.md)
