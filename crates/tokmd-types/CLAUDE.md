# tokmd-types

## Purpose

Core data structures and contracts for tokmd. This is a **Tier 0** crate with no business logic dependencies.

## Responsibility

- Pure data types for receipts, rows, and reports
- Serialization/deserialization definitions via Serde
- Schema versioning (`SCHEMA_VERSION = 2`)
- **NOT** for file I/O, CLI parsing, or complex business logic

## Public API

### Data Types
- `Totals` - Aggregate statistics (files, lines, code, comments, blanks, bytes, tokens)
- `LangRow` / `LangReport` - Language-level summaries
- `ModuleRow` / `ModuleReport` - Module/directory breakdowns
- `FileRow` / `ExportData` - File-level inventory

### Receipt Types
- `LangReceipt` - Language summary with envelope metadata
- `ModuleReceipt` - Module summary with envelope metadata
- `ExportReceipt` - File-level export with envelope metadata
- `RunReceipt` - Full scan artifact bundle
- `ContextReceipt` - LLM context packing result

### Enums
- `FileKind` - Parent or Child (embedded language)
- `ScanStatus` - Complete or Partial

### Metadata Types
- `ToolInfo` - Version and tool metadata for JSON envelopes
- `ScanArgs` - Captured scan arguments for reproducibility
- `LangArgs`, `ModuleArgs`, `ExportArgs` - Command-specific args
- `LangArgsMeta`, `ModuleArgsMeta`, `ExportArgsMeta` - Args metadata for receipts

## Implementation Details

### Deterministic Output
All types are designed for deterministic serialization:
- Use `BTreeMap` (via sorting in tokmd-model) for stable key ordering
- Consistent field ordering in structs

### Schema Versioning
```rust
pub const SCHEMA_VERSION: u8 = 2;
```
Increment when modifying JSON output structure. Update `docs/schema.json` accordingly.

### Children/Embedded Language Handling
- `ChildrenMode::Collapse` - Merge embedded languages into parent totals
- `ChildrenMode::Separate` - Show as "(embedded)" rows

## Dependencies

- `serde` with derive feature (serialization)
- `tokmd-config` (configuration types)

## Testing

- Property-based tests with `proptest`
- Serde JSON roundtrip tests
- Run: `cargo test -p tokmd-types`

## Do NOT

- Add file I/O or network calls
- Add CLI parsing logic
- Add business logic beyond data modeling
- Import higher-tier crates
