# tokmd-git

Git history and diff helpers for tokmd.

## Problem

Use this crate when you need commit history, touched files, or stable range
handling without pulling in libgit2.

## What it gives you

- `git_available` and `repo_root`
- `collect_history`
- `get_added_lines`
- `rev_exists` and `resolve_base_ref`
- `GitRangeMode` with `TwoDot` and `ThreeDot`
- `classify_intent`

## Quick use / integration notes

```toml
[dependencies]
tokmd-git = { workspace = true }
```

This crate shells out to `git`, streams log output, and keeps range handling
deterministic.

## Go deeper

Tutorial: [Root README](../../README.md)
How-to: [Recipes](../../docs/recipes.md)
Reference: [Source](src/lib.rs)
Explanation: [Architecture](../../docs/architecture.md)
