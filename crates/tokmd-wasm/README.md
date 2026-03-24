# tokmd-wasm

Browser and worker bindings for tokmd in-memory workflows.

## Problem

Run tokmd in the browser without depending on a filesystem-backed scan path.

## What it gives you

- `version`, `schemaVersion`, and `analysisSchemaVersion`
- `runJson`, `run`, `runLang`, `runModule`, `runExport`, and `runAnalyze`
- a thin `wasm-bindgen` wrapper over `tokmd-core`

## Quick use / integration notes

```json
{
  "inputs": [
    { "path": "src/lib.rs", "text": "pub fn alpha() {}\n" },
    { "path": "tests/basic.py", "text": "print('ok')\n" }
  ]
}
```

Inputs are ordered `{ path, text | base64 }` rows.

`lang`, `module`, `export`, and `analyze` are the supported browser workflows today. `analyze` currently accepts only `preset: "receipt"` or `preset: "estimate"`, and `analysisSchemaVersion()` is only exported when the `analysis` feature is enabled.

## Distribution

`tokmd-wasm` is intended to be consumed from a stable, versioned artifact in CI and releases, not from a mutable local directory.

The current release path is:

- GitHub release asset: `tokmd-wasm-<tag>.tar.gz` such as `tokmd-wasm-v1.9.0.tar.gz`
- Extract contents into `web/runner/vendor/tokmd-wasm/`

The runner expects the `web/runner/vendor/tokmd-wasm` layout with `tokmd_wasm.js` and `tokmd_wasm_bg.wasm` present, plus the `wasm-pack` companion files.

Use `schemaVersion()` only for core receipt families. Browser callers that consume `runAnalyze()` should read `analysisSchemaVersion()` instead.

## Go deeper

### Tutorial

- `../../web/runner/README.md`

### How-to

- `../../web/runner/README.md`

### Reference

- `src/lib.rs`

### Explanation

- `../../docs/architecture.md`
