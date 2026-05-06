# Browser Runner

Browser-facing tokmd entrypoint for the web and WASM lane. Provides a restricted, hardened surface for running tokmd analysis inside a browser worker backed by tokmd-wasm.

## Capabilities and Constraints

### What it gives you

- **Supported modes**: `lang`, `module`, `export`, `analyze` (with `receipt` or `estimate` presets only)
- **GitHub repo ingestion**: Public and token-authenticated repos via tree+contents APIs
- **In-memory cache**: Deterministic cache hit/miss behavior with explicit policy control (reuse, reload, no-store)
- **Progress events**: Phase-level progress during repo load and analysis execution
- **Rate-limit handling**: Retry logic with respectful retry-after, actionable error messages
- **Auth boundaries**: Token is never stored persistently, never included in logs/output, never shared across cache entries
- **User cancel**: Abort button for in-flight operations
- **Static UI**: No framework overhead; plain HTML + CSS + JavaScript

### Non-goals

- No persistent cache (IndexedDB, localStorage, Service Worker cache)
- No zipball-based repo load (tree+contents only)
- No new browser modes; WASM capability matrix is the limit
- No token persistence or auto-restore

## Quick Start

```bash
npm --prefix web/runner run build:wasm
npm --prefix web/runner test
```

Build output: `web/runner/vendor/tokmd-wasm/` (consumed by worker)

For repeatable deployments, download a release artifact:

```bash
# Extract into web/runner/vendor/tokmd-wasm/
tar xzf tokmd-wasm-v1.11.0.tar.gz -C web/runner/vendor/
```

## Contract Reference

**Full contract**: see [docs/capabilities/browser-runtime.md](../../docs/capabilities/browser-runtime.md)

Key sections:

- **Cache Policy**: modes (`reuse`, `reload`, `no-store`), key components, lifecycle
- **Progress Events**: phase definitions, message format, guarantees
- **Retry Policy**: rules for auth, rate-limit, server errors; respectful backoff
- **Auth Boundaries**: token isolation rules, storage prohibitions, UI safeguards

## Architecture

| File | Purpose |
|------|---------|
| `index.html` | Static shell; no framework |
| `main.js` | Main-thread UI, message dispatch, progress rendering |
| `worker.js` | Dedicated Web Worker; runs tokmd-wasm |
| `runtime.js` | Message validation, protocol handling, cancel tracking |
| `messages.js` | Message factories, type definitions, validators |
| `ingest.js` | GitHub API client, cache, retry logic, auth partitioning |
| `styles.css` | UI styling |

## Key APIs

### Cache Policy

```javascript
// Fetch with explicit cache policy
const inputs = await fetchGitHubRepoInputs({
  owner: "user",
  repo: "project",
  ref: "main",
  token: "ghp_...", // optional
  cachePolicy: { mode: "reload", scope: "memory" },
  limits: { maxFiles: 32, maxBytes: 750_000 }
});
```

Policy modes:

- `"reuse"` (default): Use cache or populate
- `"reload"`: Evict first, always fetch fresh
- `"no-store"`: Never cache, fetch every time

### Progress Events

Worker emits `type: "progress"` messages during long operations:

```javascript
{
  type: "progress",
  requestId: "run-12",
  phase: "cache" | "files" | "validating" | "running" | "serializing" | "complete" | "retry_wait",
  message: "...",
  current: 1,
  total: 3
}
```

### Error Handling

GitHub errors are categorized and surfaced with actionable guidance:

- `401`: Token invalid → "Check your personal access token"
- `404`: Repo not found → "Repository not found or not accessible"
- `403` + rate-limit: Primary limit → "Use a token for higher limits"
- `429` + retry-after: Secondary limit → "Retrying in Xs..."
- `5xx` / network: Server error → "Retrying with backoff..."

## GitHub Ingest Semantics

`fetchGitHubRepoInputs()` keeps an in-memory cache of GitHub tree/content loads.

**Cache key** is built from:

- `owner`, `repo`, `ref`
- `authMode` (`"anonymous"` or `"token"`)
- token-derived `authPartition` (never stores raw token)
- effective limits

**Cache lifecycle**:

- Successful loads: reused during page lifetime
- Failed loads: evicted immediately (retry-friendly)
- `clearGitHubRepoCache()`: drop all or by repo
- Page refresh: cache resets

**Concurrent requests**: Callers for the same key share the in-flight fetch. Each caller has its own `AbortSignal`; aborting one does not cancel the fetch for other waiters.

**Token isolation**: Anonymous and authenticated requests never share cache. Different tokens never share cache.

## Testing

```bash
npm --prefix web/runner test
npm --prefix web/runner run check  # type-check
```

Tests cover:

- Message validation and creation
- Cache key stability and reuse
- Cache eviction on failure
- Retry logic and rate-limit handling
- Auth partitioning and token safety
- Progress event ordering

## Validation and Release

Before shipping v1.11.0, run:

```bash
npm --prefix web/runner run check
npm --prefix web/runner test
npm --prefix web/runner run build:wasm
cargo test -p tokmd-wasm --features analysis
cargo xtask docs-check
cargo xtask proof-policy --check
```

## References

- **Tutorial**: [Root README](../../README.md)
- **How-to**: [package.json](package.json)
- **Implementation**: [worker.js](worker.js), [runtime.js](runtime.js)
- **UI**: [main.js](main.js)
- **Full contract**: [docs/capabilities/browser-runtime.md](../../docs/capabilities/browser-runtime.md)
