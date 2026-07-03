# Plan: File I/O Cache Evidence (Lane 3 PR C)

- Status: active
- Related spec: none (plan precedes implementation)
- Related ADR: none
- Related issues: Lane 3 queue item 4 in [ROADMAP.md](../ROADMAP.md)

## Goal

Decide whether a bounded read-through cache for repeated file opens in analysis
content enrichers is worth implementing. **No cache implementation in this
plan** — only measurement protocol, thresholds, invalidation rules, and claim
boundary.

## Problem statement

Several analysis enrichers under `tokmd-analysis/src/content/` and
`tokmd-analysis/src/entropy/` open the same repository files independently:

| Call site | Helper | Typical use |
| --- | --- | --- |
| `content/mod.rs` | `read_head` | TODO tags, duplicate hashing, import parsing |
| `entropy/mod.rs` | `read_head_tail` | High-entropy profiling |
| `content/complexity/` | `read_lines` / `read_text_capped` | Complexity landmarks |

A single `analyze --preset health` (or `risk`) pass can open the same path
multiple times with different byte limits. PR B removed redundant
`fs::metadata` syscalls in `tokmd-model::collect_file_rows`; repeated
`File::open` in analysis remains unmeasured.

## Measurement protocol

1. **Baseline host**: same machine class and build profile as
   [perf-smoke-baseline-2026-07.md](../ci/perf-smoke-baseline-2026-07.md).
2. **Command** (bounded analysis preset touching content enrichers):

   ```bash
   cargo xtask perf-smoke \
     --target-repo . \
     --output target/perf/analysis-health-baseline.json \
     --analysis-preset health \
     --analysis-max-files 500 \
     --analysis-max-bytes 52428800 \
     --sha "$(git rev-parse HEAD)"
   ```

3. **Repeat** on candidate commits with identical limits.
4. **Optional deep probe** (maintainer-only, not CI): add temporary trace
   counters around `content::io::read_*` to count `(path, limit)` open pairs
   per analyze run. Remove trace before merge unless promoted to a permanent
   `xtask` introspection flag.

## Decision thresholds

Proceed to a **narrow cache implementation PR** only when **all** hold on the
same host with two consecutive receipts:

| Signal | Threshold | Rationale |
| --- | --- | --- |
| `analysis_workflows[].total_ms` (health preset) | ≥ **15%** reduction potential in a prototype branch **or** ≥ **200 ms** absolute on self-scan | Below this, cache complexity dominates |
| Repeated `(path, limit)` opens | ≥ **1.3×** file count (500-file cap) | Confirms duplicate-read hypothesis |
| Row / warning parity | Identical `row_count`, `warning_count`, enabled reports | Output must not change |

If thresholds are not met, record **no-cache** outcome and defer.

## Invalidation rules (for a future implementation)

Any cache must be:

- **Request-scoped**: created at `analyze_workflow` entry, dropped on return.
- **Keyed by** `(canonical repo-relative path, max_bytes, read_mode)` where
  `read_mode` ∈ `{head, head_tail, lines, text_capped}`.
- **Invalidated** when the substrate/export file list changes mid-request
  (should not happen today; assert in debug builds).
- **Bounded**: respect existing `ContentLimits` / `AnalysisLimits`; never
  retain more bytes than the largest per-file cap in the request.
- **Not persisted** across CLI invocations, WASM workers, or sensor runs without
  a separate durability spec.

## Non-goals

- Cross-process or cross-request disk cache.
- Caching scan/tokei results (separate lane).
- Changing public receipt schemas or preset defaults.
- Implementing cache in this plan PR.

## Work Packets

1. **PR C (this plan)** — evidence protocol and thresholds. **Status: complete**
   (shipped in #406, imported @ `41c05d30`).
2. **PR D — trace-counter branch** — permanent opt-in content-open trace
   (`tokmd-analysis::io_trace` + `cargo xtask perf-smoke --trace-io`).
   **Status: complete.** Measurement receipt and threshold evaluation:
   [perf-smoke-io-trace-2026-07.md](../ci/perf-smoke-io-trace-2026-07.md).
   Outcome: duplicate-read hypothesis **confirmed** (health preset re-opens
   capped files ~1.54×, ≥ 1.3×), but the timing-reduction threshold is **not
   established** by an open count. The plan's alternate PR D option — a
   prototype request-scoped cache measured with a health-preset before/after
   A/B — remains available and is the only instrument that can settle the
   timing threshold.
3. **Future PR E** — production cache only if a prototype A/B meets **all**
   thresholds with parity tests. **Status: not started (no-go on current
   evidence).** The trace confirms duplicates but does not prove a ≥ 15% /
   ≥ 200 ms `analyze total_ms` win, so PR E stays deferred.

## Validation

```bash
cargo xtask doc-artifacts --check
cargo fmt-check
git diff --check
```

## Stop Conditions

- Stop if the plan PR adds cache implementation code.
- Stop if thresholds are cited without perf-smoke health receipts.
- Stop if docs claim cross-platform duplicate-read rates from self-scan only.
- Stop if receipt schemas or preset defaults change in the plan PR.

## Claim boundary

- **Establishes**: named measurement command, thresholds, invalidation sketch,
  and explicit no-implementation scope for PR C.
- **Does not establish**: that a cache helps, that health-preset timings improved,
  or that cross-platform duplicate-read rates match self-scan.

## See also

- [perf-smoke.md](../ci/perf-smoke.md)
- [perf-smoke-baseline-2026-07.md](../ci/perf-smoke-baseline-2026-07.md)
- [perf-smoke-io-trace-2026-07.md](../ci/perf-smoke-io-trace-2026-07.md) — PR D trace measurement + PR E decision
- PR B — `tokmd-model` metadata elimination (core workflow `model_ms`)
