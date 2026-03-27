# tokmd-model

Deterministic aggregation and receipt modeling for tokmd.

## Problem
Raw scan results are not stable enough yet for output, diffing, or receipt generation.

## What it gives you
- `collect_file_rows`
- `create_lang_report`
- `create_module_report`
- `create_export_data`
- `unique_parent_file_count`
- `normalize_path`
- `module_key`

## API / usage notes
- This crate owns aggregation, sorting, filtering, and path normalization.
- It turns `tokei::Languages` plus optional in-memory inputs into tokmd receipts.
- `src/lib.rs` shows the exact report-building helpers and invariants.

## Go deeper
- Tutorial: [tokmd README](../../README.md)
- How-to: [Recipes](../../docs/recipes.md)
- Reference: [src/lib.rs](src/lib.rs)
- Explanation: [Architecture](../../docs/architecture.md)
