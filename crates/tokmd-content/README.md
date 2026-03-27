# tokmd-content

Read, hash, and score file content for tokmd analysis.

## Problem

Use this crate when path listings are not enough and you need bounded content
sampling for tags, entropy, hashes, or function-level complexity.

## What it gives you

- `read_head`, `read_head_tail`, `read_lines`, `read_text_capped`
- `is_text_like`, `hash_bytes`, `hash_file`
- `count_tags`, `entropy_bits_per_byte`
- `complexity::analyze_functions`
- `complexity::estimate_cyclomatic_complexity`
- `complexity::estimate_cognitive_complexity`
- `complexity::analyze_nesting_depth`

## Quick use / integration notes

```toml
[dependencies]
tokmd-content = { workspace = true }
```

Use the head/tail readers for bounded sampling, then feed the bytes into
entropy or tag checks.

## Go deeper

Tutorial: [Root README](../../README.md)
How-to: [Recipes](../../docs/recipes.md)
Reference: [Source](src/lib.rs)
Reference: [Complexity module](src/complexity.rs)
Explanation: [Architecture](../../docs/architecture.md)
