# tokmd-types

Core data structures and contracts for tokmd.

## Overview

This is a **Tier 0** crate containing pure data types with no business logic dependencies. It defines the receipt schemas, row types, and enums used throughout the tokmd ecosystem.

## Installation

```toml
[dependencies]
tokmd-types = "1.3"
```

## Key Types

### Receipt Types
- `LangReceipt` - Language summary with envelope metadata
- `ModuleReceipt` - Module summary with envelope metadata
- `ExportReceipt` - File-level export with envelope metadata
- `ContextReceipt` - LLM context packing result
- `DiffReceipt` - Comparison between two receipts

### Data Types
- `LangRow` / `LangReport` - Language-level statistics
- `ModuleRow` / `ModuleReport` - Module/directory breakdowns
- `FileRow` / `ExportData` - File-level inventory
- `Totals` - Aggregate statistics (files, lines, code, bytes, tokens)

### Enums
- `TableFormat` - Output format (Md, Tsv, Json)
- `ExportFormat` - Export format (Csv, Jsonl, Json, Cyclonedx)
- `RedactMode` - Redaction level (None, Paths, All)
- `ChildrenMode` - Embedded language handling (Collapse, Separate)
- `ConfigMode` - Config file strategy (Auto, None)

## Schema Versioning

```rust
pub const SCHEMA_VERSION: u32 = 2;
```

All JSON receipts include a `schema_version` field. Breaking changes increment this version.

## Stability Policy

- **JSON consumers**: Stable. New fields have sensible defaults.
- **Rust library consumers**: Semi-stable. Use `..Default::default()` patterns for forward compatibility.

## Features

- `clap` - Enable clap derive macros for enums (optional)

## License

MIT OR Apache-2.0
