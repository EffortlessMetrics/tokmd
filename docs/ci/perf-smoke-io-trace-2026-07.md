# Perf-Smoke I/O Open Trace — 2026-07 (Lane 3, PR D)

Content-open measurement for the file-I/O cache decision described in
[file-io-cache-evidence.md](../plans/file-io-cache-evidence.md). This is the
plan's **Future PR D** *"optional trace counter"* branch: it counts
`(read_mode, max_bytes, path)` opens per analyze pass so the duplicate-read
hypothesis can be evaluated **before** any cache is implemented.

## Instrument

`tokmd-analysis::io_trace` records content opens at the `content::io` read
facade while a request-scoped trace is active. `cargo xtask perf-smoke
--trace-io` installs one scope around each timed `analyze` preset and emits an
`io_trace` section (`tokmd.io_open_trace.v1`) in the perf-smoke receipt. The
trace is thread-local and idle-free when the flag is absent; analysis runs
single-threaded, so the scope captures every open.

Counts are **open attempts** at the facade (recorded before `File::open`), so a
repeated attempt on the same key is a repeated open in the duplicate-read sense.
`read_text_capped` delegates to the inner head reader and is counted once as
`text_capped`, not as a duplicate `head` + `text_capped` pair.

## Command

```bash
cargo xtask perf-smoke \
  --target-repo . \
  --output xtask/target/perf/health-trace-run1.json \
  --analysis-preset health \
  --analysis-max-files 500 \
  --analysis-max-bytes 52428800 \
  --trace-io \
  --sha "$(git rev-parse HEAD)"
```

Build profile: **debug** `xtask` (local Windows MSVC host), matching
[perf-smoke-baseline-2026-07.md](perf-smoke-baseline-2026-07.md). Captured on the
Lane 3 branch with the PR D trace instrumentation applied, tokmd-swarm self-scan,
two consecutive receipts.

## Results (health preset, self-scan, 2026-07-02)

| Receipt | `analyze total_ms` | `total_opens` | `unique_paths` | `unique_keys` | `duplicate_key_opens` | `max_opens_for_key` | `opens_per_path` |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| run 1 | 6473 | 768 | 500 | 500 | 268 | 3 | 1.536 |
| run 2 | 6267 | 768 | 500 | 500 | 268 | 3 | 1.536 |

- All opens were `head` mode (`by_mode.head.opens = 768`, `unique_keys = 500`).
  `head_tail` / `lines` / `text_capped` did not fire under the `health` preset
  (enabled reports: `derived`, `complexity`).
- `unique_paths` sits exactly at the `--analysis-max-files 500` cap; the corpus
  is larger, so the ratio is a floor for this preset on this host.
- The open trace is **deterministic** across the two consecutive receipts.
- `warning_count = 0` in both; `row_count` was 1811 / 1812 (unrelated scan-edge
  variance, not affected by the trace).

## Decision thresholds (from the plan)

| Signal | Threshold | Observed | Met? |
| --- | --- | --- | --- |
| Repeated `(path, limit)` opens | ≥ **1.3×** file count | **1.536×** (768 / 500), stable ×2 | **yes** |
| `analyze total_ms` (health) reduction potential | ≥ **15%** or ≥ **200 ms** | not measurable from an open **count** | **not established** |
| Row / warning parity | identical rows/warnings/reports | n/a (no cache built) | n/a |

## Outcome: confirm duplicates, **do not** proceed to PR E yet

The duplicate-read hypothesis is **confirmed**: the `health` preset opens each of
the 500 capped files ~1.54× (some up to 3×), all `head` mode with an identical
byte limit — i.e. the same `(head, limit, path)` key is opened more than once per
pass.

However, the plan requires **all** thresholds to hold before a cache
implementation PR, and the timing-reduction threshold is **not established**. A
trace counter measures open *count*, not the time attributable to duplicate
opens. Each duplicate is a bounded `head` read of an already-warm small source
file (sub-millisecond class), and `analyze total_ms` (~6.3 s) is dominated by
analysis compute (complexity landmarks, derived metrics), not file opens. There
is no evidence that eliminating the 268 duplicate head-reads would reclaim ≥ 200 ms
or ≥ 15%.

Per the plan's "proceed only when **all** hold" rule, the outcome is
**no-go for PR E (production cache)** at this time. The confirmed duplicate rate
justifies keeping the lane open for the plan's *other* PR D option — a
prototype request-scoped cache measured with a health-preset before/after A/B —
which is the only instrument that can settle the timing threshold. That prototype
is a separate, still-optional PR and is intentionally **out of scope** here.

## Claim boundary

- **Establishes**: a repeatable content-open trace instrument, and that the
  `health` preset re-opens capped files ~1.54× (≥ 1.3×) on this host/corpus,
  deterministically across two consecutive receipts.
- **Does not establish**: that a read-through cache reduces `analyze total_ms`,
  any release-profile or CI wall-clock effect, or cross-platform duplicate-read
  rates. Timing-reduction potential remains unproven and requires a prototype
  cache A/B, not an open-count trace.

## See also

- [file-io-cache-evidence.md](../plans/file-io-cache-evidence.md) — plan, thresholds, invalidation sketch
- [perf-smoke.md](perf-smoke.md) — maintainer workflow and comparison rules
- [perf-smoke-baseline-2026-07.md](perf-smoke-baseline-2026-07.md) — core-workflow baseline
