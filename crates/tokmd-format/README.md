# tokmd-format

Output formatting and serialization for tokmd.

## Overview

This is a **Tier 2** crate that renders tokmd receipts to various formats: Markdown, TSV, JSON, JSONL, CSV, and CycloneDX SBOM.

## Installation

```toml
[dependencies]
tokmd-format = "1.3"
```

## Usage

```rust
use tokmd_format::{print_lang_report, scan_args, normalize_scan_input};

// Print language report to stdout
print_lang_report(&report, &global_args, &lang_args)?;

// Build ScanArgs with optional redaction
let args = scan_args(&paths, &global, Some(RedactMode::Paths));

// Normalize path for cross-platform consistency
let normalized = normalize_scan_input(&path);
```

## Supported Formats

### Table Formats
- **Markdown** - Pipes with right-aligned numeric columns
- **TSV** - Tab-separated with header row
- **JSON** - Receipt with envelope metadata

### Export Formats
- **CSV** - Standard comma-separated
- **JSONL** - Lines with type discriminator
- **JSON** - Full receipt array
- **CycloneDX 1.6** - SBOM with tokmd-specific properties

## Key Functions

### Console Output
- `print_lang_report()` - Language summary
- `print_module_report()` - Module breakdown

### File Writing
- `write_export()` - Write export data
- `write_lang_json_to_file()` - Language receipt
- `write_module_json_to_file()` - Module receipt
- `write_export_jsonl_to_file()` - Export receipt

### ScanArgs Construction
- `scan_args()` - Single source of truth for building ScanArgs with redaction

## Redaction Modes

| Mode | Behavior |
|------|----------|
| `None` | Paths shown as-is |
| `Paths` | Hash paths, preserve extensions |
| `All` | Hash paths and excluded patterns |

## Re-exports

```rust
pub use tokmd_redact::{redact_path, short_hash};
```

## License

MIT OR Apache-2.0
