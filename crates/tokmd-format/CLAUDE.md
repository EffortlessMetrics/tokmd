# tokmd-format

## Purpose

Output formatting and serialization. This is a **Tier 2** crate that renders tokmd receipts to various formats.

## Responsibility

- Render Markdown tables
- Serialize to JSON/JSONL/CSV
- Output file writing
- CycloneDX SBOM generation
- Redaction integration
- **NOT** for analysis-specific formatting (see tokmd-analysis-format)

## Public API

### Console Output
```rust
pub fn print_lang_report(report, global, args) -> Result<()>
pub fn print_module_report(report, global, args) -> Result<()>
```

### File Writing
```rust
pub fn write_export(export, global, args) -> Result<()>
pub fn write_lang_json_to_file(path, report, scan, args_meta) -> Result<()>
pub fn write_module_json_to_file(path, report, scan, args_meta) -> Result<()>
pub fn write_export_jsonl_to_file(path, export, scan, args_meta) -> Result<()>
```

### Re-exports
```rust
pub use tokmd_redact::{redact_path, short_hash};
```

## Supported Formats

### Table Formats
- **Markdown** - Pipes with right-aligned numeric columns
- **TSV** - Tab-separated with header row
- **JSON** - Receipt with envelope metadata

### Export Formats
- **CSV** - Standard comma-separated
- **JSONL** - Lines with type discriminator (`"meta"` or `"row"`)
- **JSON** - Full receipt array
- **CycloneDX 1.6** - SBOM with tokmd-specific properties

## Key Patterns

### Markdown Table Alignment
Numeric columns (files, lines, code, etc.) are right-aligned.

### JSON Receipts
Include schema_version and tool metadata:
```json
{
  "schema_version": 2,
  "tool": { "name": "tokmd", "version": "..." },
  "scan": { ... },
  "data": { ... }
}
```

### JSONL with Metadata
When `--meta` is enabled:
```jsonl
{"type":"meta","scan":{...},"args":{...}}
{"type":"row","path":"src/main.rs",...}
```

### Redaction Modes
- `None` - Paths as-is
- `Paths` - Hash paths, preserve extensions
- `All` - Hash paths and excluded patterns

## Dependencies

- `serde_json`, `csv`, `uuid`, `time`
- `tokmd-redact`, `tokmd-types`, `tokmd-config`

## Testing

```bash
cargo test -p tokmd-format
```

Tests cover:
- Snapshot tests (insta) for Markdown/TSV outputs
- Property-based tests for normalization
- Redaction mode behavior
- Metadata inclusion/exclusion

## Do NOT

- Add analysis-specific formatting (use tokmd-analysis-format)
- Add scanning logic
- Modify JSON schema without updating version
