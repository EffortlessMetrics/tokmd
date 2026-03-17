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
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use std::path::PathBuf;
use tokmd_format::{print_lang_report, scan_args, normalize_scan_input};
use tokmd_types::{LangReport, LangArgs, RedactMode, TableFormat, ChildrenMode, Totals};
use tokmd_settings::ScanOptions;

// Create dummy data
let report = LangReport {
    with_files: true,
    top: 10,
    children: ChildrenMode::Separate,
    total: Totals { code: 0, lines: 0, files: 0, bytes: 0, tokens: 0, avg_lines: 0 },
    rows: vec![],
};
let global_args = ScanOptions::default();
let lang_args = LangArgs {
    paths: vec![PathBuf::from(".")],
    format: TableFormat::Md,
    top: 10,
    files: true,
    children: ChildrenMode::Separate,
};

// Print language report to stdout
print_lang_report(&report, &global_args, &lang_args)?;

// Build ScanArgs with optional redaction
let paths = vec![PathBuf::from("src")];
let args = scan_args(&paths, &global_args, Some(RedactMode::Paths));

// Normalize path for cross-platform consistency
let normalized = normalize_scan_input(std::path::Path::new("src/lib.rs"));
assert_eq!(normalized, "src/lib.rs");
# Ok(())
# }
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
- `scan_args()` - Re-export from `tokmd-scan-args` for compatibility

## Redaction Modes

| Mode | Behavior |
|------|----------|
| `None` | Paths shown as-is |
| `Paths` | Hash paths, preserve extensions |
| `All` | Hash paths and excluded patterns |

## Re-exports

```rust,ignore
pub use tokmd_redact::{redact_path, short_hash};
```

## License

MIT OR Apache-2.0
