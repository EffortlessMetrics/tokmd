# tokmd-wasm

`tokmd-wasm` is the browser-friendly product surface for `tokmd`.

It exposes thin `wasm-bindgen` bindings over `tokmd-core`'s JSON API so browser
and worker callers can run `lang`, `module`, `export`, and `analyze` against
ordered in-memory inputs without going through the CLI.

Analyze entrypoints are intentionally narrower today: only
`preset: "receipt"` and `preset: "estimate"` are browser-safe in the wasm
wrapper. Richer analyze presets still depend on the filesystem-backed scan
path and return an error from `tokmd-wasm` until the remaining in-memory
analysis seams land. This applies consistently to `runJson("analyze", ...)`,
`run("analyze", ...)`, and `runAnalyze()`. These rootless analyze modes can
still report partial results with warnings when host-backed file or git
enrichers are unavailable in browser mode.

## Exports

- `version()`
- `schemaVersion()` for core receipts (`lang`, `module`, `export`)
- `analysisSchemaVersion()` when the `analysis` feature is enabled
- `runJson(mode, argsJson)`
- `run(mode, args)`
- `runLang(args)`
- `runModule(args)`
- `runExport(args)`
- `runAnalyze(args)` when the `analysis` feature is enabled
  Analyze currently supports only `receipt` and `estimate` across `runJson`,
  `run`, and `runAnalyze`.

## Input shape

The wrapper reuses the `tokmd-core::ffi::run_json` contract. In-memory inputs
use the same JSON shape that the Node and Python bindings already accept:

```json
{
  "inputs": [
    { "path": "src/lib.rs", "text": "pub fn alpha() {}\n" },
    { "path": "tests/basic.py", "text": "print('ok')\n" }
  ]
}
```

`runJson()` returns the raw response envelope as a JSON string. `run()` and the
fixed-mode helpers unwrap that envelope and return the `data` payload as a
plain JavaScript object, throwing on upstream errors.

Use `schemaVersion()` only for core receipt families. Browser callers that
consume `runAnalyze()` should read `analysisSchemaVersion()` instead.
