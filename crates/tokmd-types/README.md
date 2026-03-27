# tokmd-types

Core receipt and schema contracts for tokmd.

## Problem
Receipts, rows, and enums need one stable serde contract without pulling in CLI or scan logic.

## What it gives you
- Core rows and totals: `Totals`, `LangRow`, `ModuleRow`, `FileRow`
- Receipt wrappers: `LangReceipt`, `ModuleReceipt`, `ExportReceipt`, `ContextReceipt`, `DiffReceipt`, `RunReceipt`
- Shared enums and helpers: `TableFormat`, `ExportFormat`, `ConfigMode`, `ChildrenMode`, `ChildIncludeMode`, `RedactMode`, `AnalysisFormat`, `FileKind`, `ScanStatus`
- Contract markers: `SCHEMA_VERSION`, `HANDOFF_SCHEMA_VERSION`, `CONTEXT_SCHEMA_VERSION`, `CONTEXT_BUNDLE_SCHEMA_VERSION`

## API / usage notes
- Use this crate for serde-compatible receipt payloads and report rows.
- Enable the `clap` feature only if you need derive support for the exported enums.
- `src/lib.rs` is the source of truth for field names, schema versions, and wrapper shapes.

## Go deeper
- Tutorial: [tokmd README](../../README.md)
- How-to: [Recipes](../../docs/recipes.md)
- Reference: [SCHEMA](../../docs/SCHEMA.md) and [schema.json](../../docs/schema.json)
- Explanation: [Design](../../docs/design.md)
