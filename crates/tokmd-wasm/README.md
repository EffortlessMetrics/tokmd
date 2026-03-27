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

The browser runner consumes a release tarball named `tokmd-wasm-<tag>.tar.gz` and extracts it into `web/runner/vendor/tokmd-wasm/`.

## Go deeper

### Tutorial

- `../../web/runner/README.md`

### How-to

- `../../web/runner/README.md`

### Reference

- `src/lib.rs`

### Explanation

- `../../docs/architecture.md`
