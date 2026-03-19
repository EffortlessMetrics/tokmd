# tokmd-core

Clap-free library facade for embedding `tokmd` workflows in Rust or through FFI-friendly JSON calls.

## Overview

This is the **Tier 4** crate that exposes the binding-friendly workflow layer. Use it when you want deterministic receipts without depending on the CLI crate or on lower-tier scan/model internals directly.

## Installation

For core receipt workflows only:

```toml
[dependencies]
tokmd-core = "1.8"
```

If you also want analysis or cockpit workflows, enable the corresponding features:

```toml
[dependencies]
tokmd-core = { version = "1.8", features = ["analysis", "cockpit"] }
```

## Rust Workflow Example

```rust,no_run
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use tokmd_core::settings::{LangSettings, ScanSettings};
use tokmd_core::lang_workflow;

let scan = ScanSettings::current_dir();
let lang = LangSettings {
    top: 10,
    files: true,
    ..Default::default()
};

let receipt = lang_workflow(&scan, &lang)?;
assert!(!receipt.report.rows.is_empty());
# Ok(())
# }
```

With the `analysis` feature enabled, the same facade can drive the effort-aware analysis path:

```rust,no_run
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use tokmd_core::settings::{AnalyzeSettings, ScanSettings};
use tokmd_core::analyze_workflow;

let scan = ScanSettings::current_dir();
let analyze = AnalyzeSettings {
    preset: "estimate".to_string(),
    ..Default::default()
};

let receipt = analyze_workflow(&scan, &analyze)?;
assert!(receipt.effort.is_some());
# Ok(())
# }
```

## Main Workflows

- `lang_workflow(scan, lang) -> LangReceipt`
- `module_workflow(scan, module) -> ModuleReceipt`
- `export_workflow(scan, export) -> ExportReceipt`
- `diff_workflow(settings) -> DiffReceipt`
- `analyze_workflow(scan, analyze) -> AnalysisReceipt` with `analysis` feature
- `cockpit_workflow(settings) -> CockpitReceipt` with `cockpit` feature

All workflows use pure settings types from `tokmd_core::settings`, so they stay free of clap-specific argument structures.

## FFI JSON API

`tokmd-core` also exposes a single JSON entrypoint for bindings:

```rust,no_run
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use tokmd_core::ffi::run_json;

let envelope = run_json("lang", r#"{"paths":["."],"top":5}"#);
let parsed: serde_json::Value = serde_json::from_str(&envelope)?;

assert_eq!(parsed["ok"], true);
assert_eq!(parsed["data"]["mode"], "lang");
# Ok(())
# }
```

Supported `run_json` modes are `lang`, `module`, `export`, `analyze`, `diff`, `cockpit`, and `version`.

Response shape:

- Success: `{"ok": true, "data": {...}}`
- Error: `{"ok": false, "error": {"code": "...", "message": "..."}}`

## Re-exports

For convenience, the crate re-exports:

```rust,ignore
pub use tokmd_config as config;
pub use tokmd_types as types;
```

## When to Use

- Embedding `tokmd` in another Rust tool
- Reusing the receipt workflows without the CLI
- Driving the JSON envelope API from Python, Node, or another FFI layer
- Accessing `estimate` analysis or cockpit workflows from Rust with explicit feature flags

## License

MIT OR Apache-2.0
