# tokmd-export-tree

Render deterministic trees from `tokmd_types::ExportData`.

## Problem

Use this crate when nested path summaries need stable trees instead of ad hoc
recursion or order drift.

## What it gives you

- `render_analysis_tree(export)`
- `render_handoff_tree(export, max_depth)`
- lexicographically ordered siblings via `BTreeMap`
- analysis trees with file leaves
- handoff trees with directory-only nodes and depth limiting

## Quick use / integration notes

```toml
[dependencies]
tokmd-export-tree = { workspace = true }
```

Pass in `ExportData` and render the tree variant that matches your report or
bundle.

## Go deeper

Tutorial: [Root README](../../README.md)
How-to: [Recipes](../../docs/recipes.md)
Reference: [Source](src/lib.rs)
Explanation: [Architecture](../../docs/architecture.md)
