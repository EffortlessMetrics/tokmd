# tokmd-wasm

`tokmd-wasm` is the browser-friendly product surface for `tokmd`.

It exposes thin `wasm-bindgen` bindings over `tokmd-core`'s JSON API so browser
and worker callers can run `lang`, `module`, `export`, and `analyze` against
ordered in-memory inputs without going through the CLI.

## Exports

- `version()`
- `schemaVersion()`
- `runJson(mode, argsJson)`
- `run(mode, args)`
- `runLang(args)`
- `runModule(args)`
- `runExport(args)`
- `runAnalyze(args)` when the `analysis` feature is enabled

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

