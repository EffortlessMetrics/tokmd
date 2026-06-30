# Syntax Evidence for UB and Crash Review

Status: user-path guide for the shipped opt-in syntax surface.

Related specs: [`syntax-receipts.md`](../specs/syntax-receipts.md),
[`evidence-packet-workflow.md`](../specs/evidence-packet-workflow.md)

Related recipes: [`integrations/ub-review.md`](../integrations/ub-review.md),
[`packet-workflows.md`](../packet-workflows.md)

## Purpose

Use parser-backed syntax receipts when a review needs **advisory first-read
ordering** for native boundaries, panic seams, dynamic execution, and similar
crash-adjacent patterns—without promoting syntax facts onto default analysis
receipts or claiming reachability.

Syntax evidence answers:

- which scoped files parsed successfully;
- what parser-derived seams exist in those files;
- how to rank them for human or agent inspection;
- where each signal lives in `syntax.json` for follow-up.

Syntax evidence does **not** answer:

- whether undefined behavior exists or is absent;
- whether a seam is reachable at runtime;
- whether guards are sufficient;
- merge readiness or CI proof.

## Prerequisites

- A `tokmd` binary built with the default `ast` feature (plain `cargo install
  tokmd` includes it).
- Explicit opt-in: run `tokmd syntax` or `tokmd packet generate` with syntax
  enabled (syntax is on by default for packet generate; use `--no-syntax` to
  skip).
- Scoped paths: pass the same changed paths you would use for `bun-ub` or
  scoped analyze—not the whole repo unless that is the review scope.

Global `--exclude` patterns apply to syntax path collection in both `tokmd
syntax` and `tokmd packet generate`.

## Generate Evidence

### One-command packet (recommended)

```bash
tokmd packet generate \
  --preset bun-ub \
  --base origin/main \
  --head HEAD \
  --out sensors/tokmd \
  --syntax \
  src/runtime/api src/bun.js/bindings
```

This writes `sensors/tokmd/syntax.json` plus the required analyze/context
artifacts and indexes them in `manifest.json`.

### Syntax-only (custom workflows)

```bash
tokmd syntax --no-progress src/runtime/api |
  Out-File -Encoding utf8 sensors/tokmd/syntax.json   # PowerShell UTF-8

# POSIX shell
tokmd syntax --no-progress src/runtime/api > sensors/tokmd/syntax.json
```

Then index with `tokmd evidence-packet` if you need `review_priority` in the
manifest. See [`evidence-packet.md`](../evidence-packet.md).

### Exclude generated or vendor paths

```bash
tokmd packet generate \
  --exclude '**/vendor/**' \
  --exclude '**/*.gen.ts' \
  --base origin/main \
  --head HEAD \
  src/
```

Excluded paths are omitted from `syntax.json` receipts the same way as `tokmd
syntax`.

## Read `review_signals`

Each file receipt in `syntax.json` may include a `review_signals` array. Signals
are deterministic, sorted by effective score, and categorized for cross-language
ranking.

| Category | Inspect when… |
| --- | --- |
| `native_boundary` | FFI, bindings, `dlopen`, `ctypes`, or native interop changed |
| `panic_seam` | Rust `unwrap`/`expect`, indexing, assert macros, or allocation seams |
| `dynamic_execution` | `eval`, dynamic calls, or runtime-constructed behavior |
| `dynamic_import` | Runtime `import()` or equivalent |
| `process_boundary` | Subprocess or shell invocation |
| `io_boundary` | File I/O at a crash-sensitive seam |
| `exception_path` | Python raise/handler paths |
| `entrypoint` | `main`, servers, or startup hooks |
| `public_surface` | Exported or API-visible symbols in scope |
| `guard_evidence` | Nearby `if`/`match`/`try` guards that may bound a higher-risk seam |

Each signal includes `severity`, `score`, `kind`, `reason`, `evidence`, and
often a `span` (1-based line/column). Treat high scores as **read-first hints**,
not verdicts.

For `bun-ub`, test-context panic seams (`test_context: true`) are deprioritized
so production panic seams surface first. The lowered score is intentional; do not
reinterpret it as “safe in production.”

## Triage Workflow

1. **Open the manifest** — `sensors/tokmd/manifest.json`.
2. **Check status** — `complete` means required artifacts resolved; `partial`
   may still include usable syntax evidence with warnings.
3. **Read `review_priority`** — when present, this is the packet-level sorted
   view of syntax signals with JSON Pointer refs back to `syntax.json`.
4. **Walk refs** — follow each `refs` entry to the underlying receipt and span.
5. **Cross-check analyze + context** — use `analyze.md` for preset signals
   (effort, churn, imports) and `context.md` for source actually included in
   the budget.
6. **Record non-claims** — copy `non_claims` into review notes so downstream
   agents do not over-read syntax ranking as proof.

Example: list top manifest priorities (requires `jq`):

```bash
jq '.review_priority[:5] | .[] | {rank, path, category, severity, score, refs}' \
  sensors/tokmd/manifest.json
```

Example: high-severity native boundaries in syntax receipts:

```bash
jq '.receipts[] | select(.review_signals != null) |
  {path, signals: [.review_signals[] | select(.category == "native_boundary" and .severity == "high")]} |
  select(.signals | length > 0)' sensors/tokmd/syntax.json
```

## Crash-Hunting: `panic_seam_summary`

Rust receipts may include `panic_seam_summary` on `tokmd.syntax_receipt.v1`. It
derives guard status, input-source hints, and FFI/JS-arg suspects from syntax
facts for agents hunting crash stacks—not for automated fix suggestions.

Use it to narrow **where to read next** in a file already flagged by
`review_signals`. It does not prove which branch executed in production.

## Agent and Bot Consumption

- Prefer `manifest.json` → `review_priority` for ordering; drill into
  `syntax.json` via refs for spans and evidence strings.
- When syntax is unavailable (`syntax.json` missing), continue with analyze +
  context; do not infer “no native risk.”
- When `syntax.json` status is `partial` or individual receipts are advisory
  (`parse_degraded`, `skipped_too_large`, `unsupported_language`), treat signals
  from other receipts as bounded, not complete coverage.
- Never promote syntax categories to gate failures without a separate policy
  proposal; default receipts and gates remain unchanged.

## Reproduce and Verify

```bash
# Regenerate the scoped packet locally
tokmd packet generate --base origin/main --head HEAD --syntax src/runtime/api

# Validate manifest schema and artifact linkage
tokmd evidence-packet --base origin/main --head HEAD src/runtime/api
```

For maintainer shadow/compare proof of parser behavior, see
[`ast-shadow.md`](../specs/ast-shadow.md) and `cargo xtask ast-shadow-compare`.

## Non-Claims

This workflow packages **advisory parser evidence** for UB/crash-oriented review.
It does not replace sanitizers, fuzzers, valgrind, or human judgment. Syntax
ranking helps reviewers and agents spend attention; it does not certify safety.
