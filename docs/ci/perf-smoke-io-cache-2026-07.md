# Perf-Smoke I/O Cache Prototype A/B — 2026-07 (Lane 3, PR F)

Health-preset before/after A/B for the file-I/O cache decision described in
[file-io-cache-evidence.md](../plans/file-io-cache-evidence.md). This is the
plan's remaining **PR D option** — *"a prototype request-scoped cache measured
with a health-preset before/after A/B"* — which the
[PR D open trace](perf-smoke-io-trace-2026-07.md) named as the only instrument
that can settle the timing-reduction threshold. The trace confirmed duplicate
opens (~1.54×) but could not measure the *time* attributable to them; this A/B
measures it directly.

## Instrument

`tokmd-analysis::io_cache` is a request-scoped, thread-local read-through cache
prototype that mirrors the `io_trace` scope pattern. When installed it serves
repeated `(read_mode, max_bytes, path)` byte reads (`head` / `head_tail`) from a
per-request cache instead of re-opening the file. `cargo xtask perf-smoke
--cache-io` installs one cache scope around each timed `analyze` preset and emits
an `io_cache` section (`tokmd.io_cache_prototype.v1`) with lookups, hits, misses,
retained entries, and served bytes.

The cache is **scope-gated and off by default**: when no scope is active the
content read facade calls the reader directly, so the default `analyze` path is
byte-for-byte unchanged. Served bytes are an exact clone of the first read for
each key (proven by `io_cache` unit + facade parity tests), so a cached run
produces identical analysis input to an uncached run. Only the byte-returning
modes are cached; under the `health` preset every content open is `head` mode
(see the PR D trace), so the cache covers 100% of the measured duplicate reads.

## Command

```bash
# Baseline (cache off), two consecutive receipts
cargo xtask perf-smoke \
  --target-repo . \
  --output xtask/target/perf/health-cache-off-run1.json \
  --analysis-preset health \
  --analysis-max-files 500 \
  --analysis-max-bytes 52428800 \
  --sha "$(git rev-parse HEAD)"

# Prototype cache (cache on), two consecutive receipts
cargo xtask perf-smoke \
  --target-repo . \
  --output xtask/target/perf/health-cache-on-run1.json \
  --analysis-preset health \
  --analysis-max-files 500 \
  --analysis-max-bytes 52428800 \
  --cache-io \
  --sha "$(git rev-parse HEAD)"
```

Build profile: **debug** `xtask` (local Windows MSVC host), matching
[perf-smoke-baseline-2026-07.md](perf-smoke-baseline-2026-07.md) and the PR D
trace. tokmd-swarm self-scan at `14d611cb`, two consecutive receipts per arm.

## Results (health preset, self-scan, 2026-07-02)

| Receipt | `analyze total_ms` | cache `lookups` | `hits` | `misses` | `entries` | `hit_rate` |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| cache off, run 1 | 6235 | — | — | — | — | — |
| cache off, run 2 | 6447 | — | — | — | — | — |
| cache on, run 1 | 6493 | 768 | 268 | 500 | 500 | 0.349 |
| cache on, run 2 | 6175 | 768 | 268 | 500 | 500 | 0.349 |

- **cache off** mean `total_ms` ≈ **6341** (6235, 6447).
- **cache on** mean `total_ms` ≈ **6334** (6493, 6175).
- **Difference ≈ 7 ms (~0.1%)**, well inside run-to-run noise: the two cache-off
  runs alone differ by 212 ms and the two cache-on runs by 318 ms, so the arm
  means overlap entirely.
- The cache is doing exactly what the PR D trace predicted: **768 lookups, 500
  unique keys (entries), 268 hits** — i.e. it serves all 268 duplicate `head`
  opens (`hit_rate` = 268 / 768 = 0.349), matching the trace's
  `duplicate_key_opens = 268` and `total_opens = 768`.
- `warning_count = 0` in all four; `row_count` drifted 1819 → 1822 across the run
  order (the receipts are written into `xtask/target/` between runs, so each
  self-scan sees one more file). This is the same scan-edge variance noted in the
  PR D trace and is present in both arms; it is not caused by the cache, whose
  byte parity is proven by tests.

## Decision thresholds (from the plan)

| Signal | Threshold | Observed | Met? |
| --- | --- | --- | --- |
| Repeated `(path, limit)` opens | ≥ **1.3×** file count | 1.536× (768 / 500) — served as 268 cache hits | **yes** |
| `analyze total_ms` (health) reduction | ≥ **15%** or ≥ **200 ms** | **~7 ms (~0.1%)**, within noise | **no** |
| Row / warning parity | identical rows/warnings/reports | `warning_count` identical; byte parity proven; `row_count` scan-edge variance in both arms | parity holds (cache does not change bytes) |

## Outcome: **NO-GO for PR E (production cache)** — now measured, not inferred

The prototype eliminates **100% of the measured duplicate content opens** (all
268 duplicate `head` reads served from cache) and still produces **no measurable
`analyze total_ms` improvement**. This is the direct timing evidence the plan
required and the PR D trace could not provide.

The result confirms the PR D reasoning: `analyze total_ms` (~6.3 s) is dominated
by analysis compute (complexity landmarks, derived metrics), not file opens. Each
duplicate is a bounded `head` read of an already-warm small source file
(sub-millisecond class), so removing all 268 duplicates is lost in run-to-run
noise. Under the plan's "proceed only when **all** thresholds hold" rule, the
timing threshold fails and **PR E stays closed**.

The prototype ships as a permanent opt-in measurement instrument (`perf-smoke
--cache-io`), mirroring `--trace-io`, so this A/B is repeatable by maintainers
and re-checkable if the analysis compute profile changes materially. It is **not**
wired into the default `analyze` path.

## Claim boundary

- **Establishes**: a repeatable prototype request-scoped read cache and a direct
  health-preset A/B on this host/corpus showing that eliminating all confirmed
  duplicate `head` opens changes `analyze total_ms` by ~0.1% (within noise), i.e.
  below the plan's ≥ 15% / ≥ 200 ms threshold.
- **Does not establish**: any release-profile or CI wall-clock effect,
  cross-platform duplicate-read timing, or behavior for presets that exercise
  `head_tail` / `lines` / `text_capped` (the `health` preset uses only `head`).
  It does not change default `analyze` behavior, receipt schemas, or preset
  defaults.

## See also

- [file-io-cache-evidence.md](../plans/file-io-cache-evidence.md) — plan, thresholds, invalidation sketch, PR E decision
- [perf-smoke-io-trace-2026-07.md](perf-smoke-io-trace-2026-07.md) — PR D open-count trace
- [perf-smoke.md](perf-smoke.md) — maintainer workflow and comparison rules
- [perf-smoke-baseline-2026-07.md](perf-smoke-baseline-2026-07.md) — core-workflow baseline
