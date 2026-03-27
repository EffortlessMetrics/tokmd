# tokmd-analysis-imports

Language-aware import parsing and target normalization for tokmd analysis.

## Problem

You need import extraction and normalization without dragging in filesystem or
receipt orchestration.

## What it gives you

- `supports_language`
- `parse_imports`
- `normalize_import_target`

## Integration notes

- Supports Rust, JavaScript, TypeScript, Python, and Go.
- Relative targets collapse to `local`.
- Keeps the parser logic small and deterministic for higher-tier enrichers.

## Go deeper

### Reference

- [Source](src/lib.rs)

### Explanation

- [Architecture](../../docs/architecture.md)
- [Philosophy](../../docs/explanation.md)
