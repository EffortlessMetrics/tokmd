# Browser Runner Skeleton

`web/runner` is the first browser-facing slice for `tokmd` `1.9.0`.

It does not load `tokmd-wasm` yet. Instead, it locks the smallest useful
worker contract so later PRs can add wasm bootstrapping, GitHub zipball
ingestion, filtering, progress, and download flows without redesigning the
message shape again.

## Files

- `index.html`: static browser shell
- `main.js`: main-thread wiring, sample requests, and log surface
- `worker.js`: dedicated Worker entrypoint
- `runtime.js`: pure request/response logic for the worker
- `messages.js`: protocol constants and helpers
- `*.test.mjs`: Node smoke tests for protocol and runtime behavior

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
- `run` currently requires ordered in-memory `inputs` rows with `{ path, text }`
- `run` validates the request shape and responds with a placeholder
  `runner_not_wired` error until the wasm bootstrap lands
- `cancel` is reserved in the protocol but returns `cancel_unavailable` for now

## Local Smoke Test

```bash
node --test web/runner/*.test.mjs
```

## Local Preview

Serve the repo root with any static file server and open `/web/runner/`.

Examples:

```bash
python -m http.server 8080
# or
npx serve .
```
