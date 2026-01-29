# tokmd-model

## Purpose

Deterministic aggregation and receipt modeling. This is a **Tier 1** crate that transforms tokei results into tokmd receipts.

## Responsibility

- Convert tokei `Languages` → tokmd receipts
- Aggregation by language and module
- Path normalization rules
- Receipt generation
- **NOT** for scanning or output formatting

## Public API

### Report Creation
```rust
pub fn create_lang_report(languages, top, with_files, children) -> LangReport
pub fn create_module_report(languages, module_roots, module_depth, children, top) -> ModuleReport
pub fn create_export_data(languages, module_roots, module_depth, children, strip_prefix, min_code, max_rows) -> ExportData
pub fn collect_file_rows(languages, module_roots, module_depth, children, strip_prefix) -> Vec<FileRow>
```

### Path Utilities
```rust
pub fn normalize_path(path: &str, strip_prefix: Option<&str>) -> String
pub fn module_key(path: &str, module_roots: &[String], module_depth: usize) -> String
```

### Statistics
```rust
pub fn unique_parent_file_count(languages: &Languages) -> usize
pub fn avg(lines: usize, files: usize) -> usize
```

## Key Patterns

### Token Estimation
```rust
const CHARS_PER_TOKEN: usize = 4;
```
Simple heuristic: `tokens = bytes / 4`

### Children Mode
- `ChildrenMode::Collapse` - Merge embedded languages into parent totals
- `ChildrenMode::Separate` - Show as "(embedded)" rows with 0 bytes/tokens

### Deterministic Sorting
All outputs sorted by:
1. Code lines (descending)
2. Name (ascending)

Uses `BTreeMap` for stable key ordering.

### Path Normalization
- Convert to forward slashes (`/`) regardless of OS
- Strip leading `./` and `/`
- Handle prefix stripping for relative paths

### Module Key Computation
- Root files → `"(root)"`
- First directory → module name
- Nested paths → take up to `module_depth` directories
- Matches against `module_roots` for custom grouping

## Dependencies

- `tokei` (Languages type)
- `serde`, `anyhow`
- `tokmd-config`, `tokmd-types`

## Testing

```bash
cargo test -p tokmd-model
```

Tests cover:
- Property-based tests (proptest) for fold operations
- Module key computation (root, crates/foo patterns, depth)
- Path normalization and stripping
- Fold associativity and empty set handling

## Do NOT

- Perform scanning (use tokmd-scan)
- Format output (use tokmd-format)
- Add CLI parsing logic
