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
pub fn scan_workflow(
    global: &GlobalArgs,
    lang: &LangArgs,
    redact: Option<RedactMode>,
) -> Result<LangReceipt>
```

### Re-exports
```rust
pub use tokmd_config as config;
pub use tokmd_types as types;
```

## Implementation Details

### Workflow

The `scan_workflow` function chains:
1. **Scan** (tokmd-scan) - Execute tokei
2. **Model** (tokmd-model) - Aggregate results
3. **Receipt** - Construct with envelope metadata

```rust
let global = GlobalArgs::default();
let lang = LangArgs {
    paths: vec![],
    format: TableFormat::Json,
    // ...
};
let receipt = scan_workflow(&global, &lang, Some(RedactMode::Paths))?;
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
- `tokmd-redact` - Path hashing
- `tokmd-config` - Args types
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
