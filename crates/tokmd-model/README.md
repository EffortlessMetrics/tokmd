# tokmd-model

Deterministic aggregation and receipt modeling for tokmd.

## Overview

This is a **Tier 1** crate containing the core business logic for transforming raw tokei scan results into tokmd receipts. It handles aggregation, sorting, path normalization, and filtering.

## Installation

```toml
[dependencies]
tokmd-model = "1.2"
```

## Usage

```rust
use tokmd_model::{create_lang_report, create_module_report, collect_file_rows};
use tokmd_types::ChildrenMode;

// Create language summary
let report = create_lang_report(&languages, 10, false, ChildrenMode::Collapse);

// Create module breakdown
let module_report = create_module_report(
    &languages,
    &["crates".to_string()],
    2,
    ChildIncludeMode::Separate,
    0
);

// Collect file-level rows
let rows = collect_file_rows(
    &languages,
    &["src".to_string()],
    1,
    ChildIncludeMode::Separate,
    None
);
```

## Key Functions

### Report Creation
- `create_lang_report()` - Aggregate by language
- `create_module_report()` - Aggregate by directory structure
- `create_export_data()` - File-level inventory with filtering
- `collect_file_rows()` - Raw file row collection

### Path Utilities
- `normalize_path()` - Cross-platform path normalization
- `module_key()` - Compute module key from path

## Key Patterns

### Token Estimation
```rust
const CHARS_PER_TOKEN: usize = 4;
```
Simple heuristic: `tokens = bytes / 4`

### Deterministic Sorting
All outputs sorted by:
1. Code lines (descending)
2. Name (ascending)

### Children Mode
- `Collapse` - Merge embedded languages into parent
- `Separate` - Show as distinct rows

### Path Normalization
- Forward slashes on all platforms
- Strip leading `./` and `/`
- Optional prefix stripping

## License

MIT OR Apache-2.0
