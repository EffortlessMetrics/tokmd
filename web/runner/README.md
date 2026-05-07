# Browser Runner

Browser-facing tokmd entrypoint for the web and WASM lane.

## Problem

Use this project when you want `tokmd` inside a browser worker, backed by
`tokmd-wasm`, without widening the browser contract to the full native CLI.

## What it gives you

- a static browser shell in `index.html`
- main-thread wiring in `main.js`
- a dedicated worker in `worker.js`
- runtime validation and protocol handling in `runtime.js`
- public GitHub repo ingestion through browser-safe tree + contents APIs
- `lang`, `module`, `export`, and `analyze` with `receipt` or `estimate`
- worker run progress events for `start`, `scan` or `analyze`, `done`, and `error`
- visible run-progress and repo-load-progress panels in the browser shell
- session-only GitHub token UX with explicit clear behavior
- explicit GitHub rate-limit retry guidance and a manual repo-load retry action
- `cancel` reserved in the protocol but not wired yet
- live result panes with downloadable JSON artifacts

## Quick use / integration notes

```bash
npm --prefix web/runner run build:wasm
npm --prefix web/runner test
```

The browser bundle loads `web/runner/vendor/tokmd-wasm` and expects the wasm
package layout produced by the build script.

## Distribution artifact

For repeatable deployments, consume a versioned release artifact from GitHub and extract it into:

```text
web/runner/vendor/tokmd-wasm/
```

The release asset is named:

```text
tokmd-wasm-<tag>.tar.gz
```

`v1.9.0` becomes `tokmd-wasm-v1.9.0.tar.gz`. Extracting this archive into `vendor/tokmd-wasm` gives the exact layout expected by `web/runner/worker.js` without rebuilding from source.

## GitHub ingest cache semantics

`fetchGitHubRepoInputs()` keeps an in-memory cache of in-flight and completed GitHub tree/content loads.

The cache key is a stable JSON value built from:

- `owner`
- `repo`
- `ref`
- `authMode` (`"anonymous"` or `"token"`)
- token-derived `authPartition`
- effective limits (`maxFiles`, `maxBytes`, `maxFileBytes`)

The token-derived auth partition prevents authenticated fetches with different tokens from sharing entries without storing the raw token in the key. Anonymous requests use the anonymous partition.

Cache lifecycle:

- Successful loads remain in memory and can be reused for equivalent requests during the page lifetime.
- Failed loads are evicted so a later retry can fetch again.
- `clearGitHubRepoCache()` drops all entries.
- Page refreshes and new browser processes start with an empty cache.

Concurrent callers for the same key share the in-flight network load. Each waiter keeps its own `AbortSignal`, so canceling one waiter does not cancel the shared load for other waiters.

## Go deeper

Tutorial: [Root README](../../README.md)
How-to: [package.json](package.json)
Reference: [worker.js](worker.js) and [runtime.js](runtime.js)
Explanation: [main.js](main.js)
