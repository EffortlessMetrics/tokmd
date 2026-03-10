# tokmd-core

High-level API facade for tokmd.

## Overview

This is a **Tier 4** crate providing the recommended entry point for library usage. It coordinates scanning, aggregation, and modeling to produce code inventory receipts.

## Installation

```toml
[dependencies]
tokmd-core = "1.7"
tokmd-types = "1.7"
tokmd-settings = "1.7"
```

## Usage

```rust,no_run
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use tokmd_core::lang_workflow;
use tokmd_core::settings::{ScanSettings, LangSettings};
use tokmd_core::types::RedactMode;

// Configure scan
let scan = ScanSettings::current_dir();
let mut lang = LangSettings {
    top: 10,
    files: false,
    ..Default::default()
};

// Run pipeline (without redaction)
let receipt = lang_workflow(&scan, &lang)?;
println!("Scanned {} languages", receipt.report.rows.len());

// Run pipeline (with path redaction)
lang.redact = Some(RedactMode::Paths);
let redacted = lang_workflow(&scan, &lang)?;
# Ok(())
# }
```

## Main Function

```rust,ignore
pub fn lang_workflow(
    scan: &ScanSettings,
    lang: &LangSettings,
) -> Result<LangReceipt>
```

Chains: Scan -> Model -> Receipt

## Redaction Modes

| Mode | Behavior |
|------|----------|
| `None` | Paths shown as-is |
| `Paths` | Hash file paths, preserve extension |
| `All` | Hash paths and excluded patterns |

## Re-exports

```rust,ignore
pub use tokmd_settings as settings;
pub use tokmd_types as types;
```

## When to Use

- Library consumers wanting a simple scan API
- Embedding tokmd in other Rust tools
- Programmatic access without CLI overhead

## License

MIT OR Apache-2.0
