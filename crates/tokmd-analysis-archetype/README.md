# tokmd-analysis-archetype

Infers a coarse repository archetype from export metadata.

## Problem
You need a quick repository-shape signal without adding a separate classification layer.

## What it gives you
- `detect_archetype`
- `Archetype`
- Heuristics for Rust workspaces, Next.js apps, containerized services, IaC projects, Python packages, and Node packages

## Integration notes
- No feature flags.
- Works from normalized export metadata, not file content.
- Designed to feed the `archetype` analysis preset.

## Go deeper
- Tutorial: [Tutorial](../../docs/tutorial.md)
- How-to: [Recipes](../../docs/recipes.md)
- Reference: [Architecture](../../docs/architecture.md), [Root README](../../README.md)
- Explanation: [Explanation](../../docs/explanation.md)
