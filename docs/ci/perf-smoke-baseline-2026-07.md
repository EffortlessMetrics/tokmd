# Perf-Smoke Baseline — 2026-07 (Lane 3)

Lane 3 optimization PRs compare against this maintainer baseline captured on
`tokmd-swarm` @ `772041af643854fd5d7ed76baa41a47aa1a90a0b` after import #2801.

## Command

```bash
cargo xtask perf-smoke \
  --target-repo . \
  --output target/perf/baseline-772041af.json \
  --sha 772041af643854fd5d7ed76baa41a47aa1a90a0b
```

Build profile: **debug** `xtask` (local Windows MSVC host). Use the same profile
for before/after comparisons on this machine; prefer `--release` when claiming
cross-session product-facing wins (see [perf-smoke.md](perf-smoke.md)).

## Baseline timings (self-scan, 2026-07-02)

| Workflow | `total_ms` | `scan_ms` | `model_ms` | `receipt_ms` | Rows | Languages |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `lang` | 2217 | 2152 | 65 | 0 | 18 | 19 |
| `module` | 4297 | 3841 | 456 | 0 | 37 | 19 |
| `export` | 6287 | 5910 | 377 | 0 | 3155 | 19 |

Receipt schema: `tokmd.perf_smoke.v1` / version `2`. Raw JSON is ephemeral under
`target/perf/` and is **not** checked in (see [perf-smoke.md](perf-smoke.md)).

## Interpretation

- **`scan_ms` dominates** core workflows on this corpus; model/receipt phases are
  secondary but measurable for export (`model_ms` ≈ 377 ms with 3155 file rows).
- **`export`** is the primary workflow for row-collection hot-path work in
  `tokmd-model` (`collect_file_rows`).
- Repeat the same command on a candidate commit; compare `model_ms` and `total_ms`
  per workflow with identical flags and corpus.

## Claim boundary

- **Establishes**: local debug-profile timings for the tokmd-swarm self-scan at
  one commit on one maintainer host class.
- **Does not establish**: release-profile SLA, CI wall-clock improvement, or
  cross-platform stability.

## See also

- [perf-smoke.md](perf-smoke.md) — maintainer workflow and comparison rules
- [ROADMAP.md](../ROADMAP.md) — Lane 3 measured performance criteria
