# Browser Runner

`web/runner` is the first browser-facing slice for `tokmd` `1.9.0`.

It now boots `tokmd-wasm` inside a dedicated worker and runs the in-memory
`lang`, `module`, `export`, and rootless `analyze` (`receipt` / `estimate`)
paths locally in the browser. Public GitHub repo acquisition now uses the
browser-safe GitHub tree and contents APIs to materialize ordered in-memory
inputs before dispatching into the worker. Zipball fetch, progress, and cancel
still come later, but the worker contract and wasm bootstrap are live now.

## Files

- `index.html`: static browser shell
- `main.js`: main-thread wiring, sample requests, and log surface
- `worker.js`: dedicated Worker entrypoint
- `runtime.js`: request validation plus mode dispatch into the wasm runner
- `messages.js`: protocol constants and helpers
- `*.test.mjs`: Node smoke tests for protocol and runtime behavior
- `package.json`: repeatable local scripts for building the browser wasm bundle

## Protocol

Worker -> main:

- `ready`
- `result`
- `error`

Main -> worker:

- `run`
- `cancel`

Current behavior:

- `ready` advertises supported modes plus `receipt` / `estimate` analyze presets
  and reports the loaded `tokmd-wasm` engine version
- `run` currently requires ordered in-memory `inputs` rows with `{ path, text }`
- the page can fetch a public GitHub `owner/repo@ref`, filter browser-unsafe
  files, and materialize `inputs` rows directly into the request editor
- `run` validates the request shape and executes the corresponding `tokmd-wasm`
  entrypoint
- `cancel` is reserved in the protocol but returns `cancel_unavailable` for now
- the page renders the capability block explicitly instead of leaving it as raw
  worker noise
- the latest successful result is shown in a dedicated artifact pane and can be
  downloaded as JSON from the browser

## Build The Browser Bundle

```bash
npm --prefix web/runner run build:wasm
```

## Local Smoke Test

```bash
npm --prefix web/runner test
```

## Local Preview

Build the browser bundle, then serve the repo root with any static file server
and open `/web/runner/`.

Examples:

```bash
npm --prefix web/runner run build:wasm
python -m http.server 8080
# or
npx serve .
```
