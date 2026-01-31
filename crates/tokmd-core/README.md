# tokmd-core

High-level API facade for tokmd.

## Overview

This is a **Tier 4** crate providing the recommended entry point for library usage. It coordinates scanning, aggregation, and modeling to produce code inventory receipts.

## Installation

```toml
[dependencies]
tokmd-core = "1.3"
tokmd-types = "1.3"
```

## Usage

```rust
use tokmd_core::scan_workflow;
use tokmd_core::config::GlobalArgs;
use tokmd_core::types::{ChildrenMode, LangArgs, RedactMode, TableFormat};
use std::path::PathBuf;

// Configure scan
let global = GlobalArgs::default();
let lang = LangArgs {
    paths: vec![PathBuf::from(".")],
    format: TableFormat::Json,
    top: 10,
    files: false,
    children: ChildrenMode::Collapse,
};

// Run pipeline (without redaction)
let receipt = scan_workflow(&global, &lang, None)?;
println!("Scanned {} languages", receipt.report.rows.len());

// Run pipeline (with path redaction)
let redacted = scan_workflow(&global, &lang, Some(RedactMode::Paths))?;
```

## Main Function

```rust
pub fn scan_workflow(
    global: &GlobalArgs,
    lang: &LangArgs,
    redact: Option<RedactMode>,
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

```rust
pub use tokmd_config as config;
pub use tokmd_types as types;
```

## When to Use

- Library consumers wanting a simple scan API
- Embedding tokmd in other Rust tools
- Programmatic access without CLI overhead

## License

MIT OR Apache-2.0
