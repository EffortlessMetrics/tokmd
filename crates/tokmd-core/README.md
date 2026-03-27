# tokmd-core

Primary library facade for embedding tokmd workflows.

## Problem

You want tokmd receipts from Rust or FFI without depending on clap or the CLI binary.

## What it gives you

- Settings-based workflows: `lang_workflow`, `module_workflow`, `export_workflow`, `diff_workflow`, `analyze_workflow`, `cockpit_workflow`
- Ordered in-memory variants: `*_workflow_from_inputs`
- Shared JSON entrypoint: `ffi::run_json`
- Convenience re-exports: `config`, `types`, `InMemoryFile`

## Quick use / integration notes

Add only the features you need:

```toml
[dependencies]
tokmd-core = { version = "1", features = ["analysis", "cockpit"] }
```

`run_json` supports these modes: `lang`, `module`, `export`, `analyze`, `diff`, `cockpit`, and `version`.

For browser-safe in-memory inputs, pass ordered `scan.inputs` rows with `{ path, text | base64 }`.

## Go deeper

### Tutorial

- `../../docs/tutorial.md`

### How-to

- `../../docs/recipes.md`

### Reference

- `src/lib.rs`
- `src/ffi.rs`
- `../../docs/SCHEMA.md`

### Explanation

- `../../docs/architecture.md`
- `../../docs/design.md`
