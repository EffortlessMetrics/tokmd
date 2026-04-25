# tokmd-core

## Purpose

High-level API facade and recommended entry point for library usage. This is a **Tier 4** coordination crate.

## Responsibility

- Coordinate scanning, aggregation, and modeling
- Provide simplified interface for library users
- Handle redaction at the top level
- **NOT** the CLI binary (see tokmd crate)

## Public API

```rust
pub fn lang_workflow(scan: &ScanSettings, lang: &LangSettings) -> Result<LangReceipt>;
pub fn module_workflow(scan: &ScanSettings, module: &ModuleSettings) -> Result<ModuleReceipt>;
pub fn export_workflow(scan: &ScanSettings, export: &ExportSettings) -> Result<ExportReceipt>;
pub fn diff_workflow(settings: &DiffSettings) -> Result<DiffReceipt>;
#[cfg(feature = "analysis")]
pub fn analyze_workflow(scan: &ScanSettings, analyze: &AnalyzeSettings) -> Result<AnalysisReceipt>;
pub fn version() -> &'static str;
```

### Re-exports
```rust
pub use tokmd_types as types;
```

### Compatibility API (deprecated)

The `scan_workflow` workflow and `tokmd_config` re-export exist for backwards compatibility
when compiled with:

```rust
#[cfg(feature = "legacy-cli")]
```

## Implementation Details

### Workflow

The settings-first workflow chain is:
1. **Scan** (tokmd-scan) - Execute tokei
2. **Model** (tokmd-model) - Aggregate results
3. **Receipt** - Construct with envelope metadata

```rust
let scan = ScanSettings::default();
let lang = LangSettings {
    top: 10,
    files: true,
    // ...
};
let receipt = lang_workflow(&scan, &lang)?;
```

### Redaction Support

| Mode | Behavior |
|------|----------|
| `None` | Paths shown as-is |
| `Paths` | Hash file paths, preserve extension |
| `All` | Hash paths and excluded patterns |

### Use Cases

- Library consumers who want a simple scan API
- Embedding tokmd in other tools
- Programmatic access without CLI

## Dependencies

- `tokmd-scan` - Tokei wrapper
- `tokmd-model` - Aggregation
- `tokmd-format::scan_args` - CLI/path/scan-arg rendering
- `tokmd-format` - Serialization + text/file output helpers
- `tokmd-settings` - Stable settings models
- `tokmd-types` - Receipt types
- `anyhow`

## Testing

```bash
cargo test -p tokmd-core
```

## Do NOT

- Add CLI parsing (use tokmd crate)
- Add formatting logic (use tokmd-format)
- Duplicate logic from lower-tier crates
