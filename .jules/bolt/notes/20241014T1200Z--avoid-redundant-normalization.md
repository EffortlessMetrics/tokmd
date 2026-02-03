# Avoid Redundant Path Normalization

**Date**: 2024-10-14T12:00:00Z
**Pattern**: Reuse already-normalized paths from intermediate structures.

## Context
In `create_module_report`, we aggregated file stats into `FileRow`s (which involved path normalization) and then re-iterated the raw source data (Tokei reports) to count unique files, repeating the normalization and key generation.

## Pattern
When an intermediate aggregation (like `Vec<FileRow>`) contains the necessary keys (path, module), reuse it for secondary aggregations instead of going back to the raw source.

## Evidence
`crates/tokmd-model/src/lib.rs`
- Before: Iterated `languages` -> `normalize_path` -> `module_key`
- After: Iterated `file_rows` (already normalized)

## Prevention
- Check if an existing collection already has the computed keys you need.
- Avoid calling `normalize_path` or `Cow` creation inside loops if the data is available elsewhere.
