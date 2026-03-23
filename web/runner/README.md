# Browser Runner

`web/runner` is the browser-facing `tokmd` slice for the current `1.9.0` lane.

It now boots `tokmd-wasm` inside a dedicated worker and runs the in-memory
`lang`, `module`, `export`, and rootless `analyze` (`receipt` / `estimate`)
paths locally in the browser. Public GitHub repo acquisition now uses the
browser-safe GitHub tree and contents APIs to materialize ordered in-memory
inputs before dispatching into the worker. Zipball fetch remains out of the
current browser contract. Repo-load progress and cancel now exist in the main
thread loader, but worker run cancel still remains intentionally unavailable.

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
- public repo fetches default to unauthenticated browser GitHub API calls, but
  the page also accepts an optional token for higher limits or private access
- repo ingestion surfaces memory-cache hits, partial-load reasons, tree
  truncation, and primary/secondary rate-limit failures explicitly
- analyze presets beyond `receipt` / `estimate` are intentionally rejected in
  browser mode instead of degrading silently
- `run` validates the request shape and executes the corresponding `tokmd-wasm`
  entrypoint
- repo-load cancel uses `AbortController`; worker `cancel` is still reserved in
  the protocol and returns `cancel_unavailable`
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
