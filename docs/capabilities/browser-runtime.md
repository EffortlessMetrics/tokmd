# Browser Runtime Capabilities

## Overview

The browser runner can load public or token-authenticated GitHub repositories through tree+contents APIs, report deterministic cache hit/miss behavior, show repo-load and worker-run progress events, handle GitHub retry/rate-limit responses with actionable UX, and prevent authenticated data or tokens from crossing cache/log/output boundaries.

This document defines the contract for v1.11.0 and beyond.

## Supported Modes and Features

The browser runner supports a restricted subset of the native CLI surface:

- **Modes**: `lang`, `module`, `export`, `analyze` (with `receipt` or `estimate` presets only)
- **Repo ingestion**: Public and token-authenticated GitHub repositories via tree+contents APIs
- **Abort support**: User-initiated cancellation of in-flight operations
- **Progress events**: Phase-level progress reporting during repo load and analysis execution
- **Cache**: In-memory cache only (no persistent storage)
- **Auth**: Token-based GitHub authentication with strict boundary enforcement

### Non-Goals

- No persistent cache (IndexedDB, localStorage, ServiceWorker)
- No zipball-based primary load path (tree+contents only)
- No expansion of browser modes beyond the WASM capability matrix
- No token persistence or storage outside of memory

---

## Cache Policy Contract

### API Surface

```javascript
cachePolicy: {
  mode: "reuse" | "reload" | "no-store",
  scope: "memory"
}
```

### Modes and Behavior

| Mode | Behavior |
|------|----------|
| `reuse` | Default. Use existing in-memory cache entry or populate one if not present. |
| `reload` | Evict matching cache entry first, then fetch fresh content and store the result. |
| `no-store` | Do not read from cache and do not write to cache. Fetch every time. |

### Cache Key Components

The cache key is built from:

- `owner` — Repository owner (username or organization)
- `repo` — Repository name
- `ref` — Git reference (branch, tag, or commit SHA)
- `authMode` — `"anonymous"` or `"token"`
- `authPartition` — Token-derived partition (never exposes raw token)
- `limits` — Effective request limits (maxFiles, maxBytes, maxFileBytes)

### Cache Metadata

Cache entries include:

- `keyVersion: 1` — Schema version for future compatibility
- `policy: { mode, scope }` — Policy that produced this entry
- `scope: "memory"` — Cache is always in-memory
- `hit: boolean` — Whether this entry was a cache hit
- `authScope: "anonymous" | "token-scoped"` — Auth level of this entry
- All metadata must never include raw token, token hash, or token-derived partition values in user-visible output

### Cache Lifecycle

- **Successful loads**: Remain in memory and can be reused for equivalent requests during the page lifetime
- **Failed loads**: Evicted immediately, allowing retry
- **Concurrent requests**: Callers for the same key share the in-flight network load; each caller keeps its own `AbortSignal`, and aborting one caller does not cancel the shared fetch
- **Clear API**: `clearGitHubRepoCache()` drops all entries or entries for a specific repository
- **Page lifecycle**: Page refresh or new browser tab starts with an empty cache

### Cache Hit Safety

On cache hits, the caller receives a defensive deep clone of cached inputs. Callers cannot mutate cached entries through the returned reference.

### Token Partitioning

- Anonymous and token-authenticated fetches never share cache entries
- Two different authentication tokens never share cache entries
- Token-derived partition keys prevent storing raw tokens while maintaining per-token cache isolation

---

## Progress Event Contract

### Message Format

Progress events are emitted as JSON objects to inform the user about long-running operations (repo loads, analysis execution).

```javascript
{
  type: "progress",
  requestId: "run-12",
  event: "tokmd.progress",
  schema_version: 1,
  kind: "update",
  phase: "validating" | "running" | "serializing" | "complete" | "cache" | "files" | "retry_wait",
  message: "Running analyze estimate...",
  current: 1,
  total: 3
}
```

### Phase Definitions

| Phase | Meaning |
|-------|---------|
| `validating` | Preparing inputs and validating arguments |
| `running` | Executing the tokmd operation (WASM call in progress) |
| `serializing` | Formatting output data |
| `complete` | Operation finished successfully |
| `cache` | GitHub repository load in progress |
| `files` | File ingestion and filtering in progress |
| `retry_wait` | Waiting before retrying a failed GitHub request |

### Fields

- `type`: Always `"progress"`
- `requestId`: Unique identifier for the operation (matches `RUN` message requestId)
- `event`: Always `"tokmd.progress"`
- `schema_version`: Grammar version, currently `1`
- `kind`: Always `"update"` for v1.11.0
- `phase`: Current operation phase
- `message`: Human-readable status message
- `current`: Optional. Current progress value (e.g., file count, step number)
- `total`: Optional. Total expected value (e.g., max file count, total steps)

### Progress Guarantees

- Phase-level progress is guaranteed during repo load and before/after WASM execution
- True per-file internal progress during Rust execution is not available in v1.11.0 (would require WASM callback/yield seam)
- Events are ordered and emit in logical sequence: validating → cache/files (if needed) → running → serializing → complete
- Abort during any phase emits a final `failed` error (not a progress event)

---

## Retry and Rate-Limit Policy

### Retry Configuration

```javascript
retryPolicy: {
  maxAttempts: 3,
  baseDelayMs: 500,
  maxDelayMs: 8000,
  retryAfterCapMs: 30000,
  respectRetryAfter: true
}
```

### Retry Rules

| Case | Retry Behavior |
|------|---|
| `401` (auth required/rejected) | **No retry**. Token is invalid or missing. Show auth guidance. |
| `404` (repo/ref not found) | **No retry**. Repository, reference, or private access unavailable. |
| Primary GitHub rate limit (`403` + `x-ratelimit-remaining: 0`) | **No auto-retry**. Show reset time. User must wait or provide token. |
| Secondary rate limit (`429` or secondary limit message) | **Retry if `Retry-After` is present and within cap**. Bounded exponential backoff. |
| `502`, `503`, `504` (server error) | **Retry with bounded exponential backoff + jitter**. |
| Network timeout or transient error | **Retry with bounded exponential backoff + jitter**. |
| Abort during retry wait | **Stop immediately**. Emit `repo_load_aborted` error. Do not complete retry attempt. |

### Retry Progress Events

During retry wait, emit progress events:

```javascript
{
  phase: "retry_wait",
  attempt: 2,
  maxAttempts: 3,
  retryAfterSeconds: 12,
  message: "GitHub asked the browser runner to slow down. Retrying in 12s..."
}
```

### Rate-Limit Headers

Respect these GitHub headers:

- `x-ratelimit-limit` — Total requests allowed in window
- `x-ratelimit-remaining` — Requests remaining in window
- `x-ratelimit-reset` — Unix timestamp of window reset
- `retry-after` — Seconds to wait before retry (absolute or relative)

---

## Authenticated Fetch Boundaries

### Hardening Rules

The browser runner implements strict boundaries around authentication to prevent token leakage:

| Boundary | Rule |
|----------|------|
| **Token in logs** | Token is never written to console logs, error messages, or debug output. Error messages may say "token rejected" but must not echo token text. |
| **Token in results** | Token is never included in result JSON, downloadable artifacts, or output data. |
| **Token in cache metadata** | Token hash and token-derived partition are never exposed in cache metadata shown to the user. |
| **Token storage** | Token is never stored in `localStorage`, `sessionStorage`, IndexedDB, cookies, URL parameters, or any persistent medium. |
| **Token in downloads** | Downloadable result artifacts never contain token or auth state. |
| **Cache isolation** | Anonymous and authenticated fetches never share cache entries. Different tokens never share entries. |

### UI Safeguards

- Token field is password-style (masked input) and non-persistent
- Auth display shows `anonymous` or `token-scoped`, never token hash
- Optional "clear token" action removes from memory
- Auth boundary notice: "Token stays in memory for this page session."

### Error Message Safety

Valid error messages:

- "GitHub rejected the token. Check your personal access token."
- "Rate limit exceeded. Use a personal access token for higher limits."
- "Repository not found or not accessible with current auth."

Invalid error messages:

- (Any that include raw token or token hash)
- (Any that quote back user input containing token)

---

## GitHub API Interaction

### Supported APIs

- **Repository contents**: `GET /repos/{owner}/{repo}/contents/{path}`
- **Repository tree**: `GET /repos/{owner}/{repo}/git/trees/{tree_sha}?recursive=1`
- **Rate limit status**: Checked via response headers

### Authentication

- **Anonymous**: Shared GitHub API limit (60 requests per hour per IP)
- **Token-authenticated**: Personal access token scoped limit (5000 requests per hour per token)

### Request Headers

For token auth, add `Authorization: token <token>` to all GitHub requests.

---

## Change Log

### v1.11.0

- Initial contract definition
- Explicit cache policy API (reuse, reload, no-store)
- Progress event protocol (phase-level)
- Retry/rate-limit engine with respectful retry-after handling
- Strict authenticated fetch boundaries
- In-memory cache only
