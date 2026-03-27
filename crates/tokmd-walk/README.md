# tokmd-walk

Deterministic filesystem traversal and asset discovery for tokmd.

## Problem

Use this crate when you need repo file listing, license candidate detection, or
file sizes without reimplementing gitignore-aware walking.

## What it gives you

- `list_files` and `list_files_from_memfs`
- `license_candidates`
- `file_size` and `file_size_from_memfs`
- deterministic alphabetical ordering
- `git ls-files` first, `ignore` fallback second

## Quick use / integration notes

```toml
[dependencies]
tokmd-walk = { workspace = true }
```

Use it for repo-local file discovery in analysis code, tests, or WASM-backed
virtual filesystems.

## Go deeper

Tutorial: [Root README](../../README.md)
How-to: [Recipes](../../docs/recipes.md)
Reference: [Source](src/lib.rs)
Explanation: [Architecture](../../docs/architecture.md)
