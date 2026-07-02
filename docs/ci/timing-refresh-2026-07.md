# CI Timing Refresh — July 2026

Lane 3 evidence for refreshing static LEM floors and stale budgeting examples
against hosted `ci-actuals` measurements.

## Why refresh now

- `policy/ci-lane-whitelist.toml` was last updated **2026-05-07**, before gate
  consolidation (#226 phase 2, #299 phase 3) retired separate Quality Gate,
  Proof Policy, and Build & Test (Linux) lanes into `tokmd_rust_result`.
- `docs/ci/lem-budgeting.md` and `docs/ci/default-pr-gate.md` still cite
  retired lane names and pre-consolidation totals (~93–203 LEM).
- Static `base_lem` floors dominate PR Plan estimates because the learned-estimate
  cache has only a handful of `main` samples; floors must reflect post-consolidation
  reality or PR Plan over-warns.

## Evidence sources

| Run | Event | SHA | Run ID | Receipt |
| --- | --- | --- | --- | --- |
| Main push | `push` → `main` | `36fbe23d` | `28607897038` | `target/ci/actuals-cache/ci-actuals.json` |
| PR merge | `pull_request` | `…b318c15f29c` | `28607236312` | `target/ci/actuals-cache-pr/ci-actuals.json` |

Both receipts are `tokmd.ci_actuals.v3`. Reproduce locally:

```bash
gh run download <RUN_ID> --repo EffortlessMetrics/tokmd-swarm \
  --name ci-actuals --dir target/ci/actuals-cache
cargo xtask ci-plan --base origin/main --head HEAD \
  --actuals-dir target/ci/actuals-cache \
  --json-out target/ci/ci-plan-learned.json
```

## Measured vs static floors (core CI jobs)

Durations are wall-clock minutes from `duration_minutes`. LEM uses
`ubuntu_latest = 1.0`; `tokmd_rust_result` ran on self-hosted (multiplier 1.0).

| Lane | Static floor | Main push | PR (#401) | Notes |
| --- | ---: | ---: | ---: | --- |
| `tokmd_rust_result` | 25 | 5.4 | 8.8 | Gate: gate+test+proof-policy concurrent |
| `feature_boundaries` | 10 | 2.1 | 2.0 | Stable ~2 min |
| `docs_check` | 4 | 1.0 | 0.8 | |
| `ci_detect_risk_packs` | 1 | 1.5 | 1.2 | Floor was below measured |
| `msrv_check` | 5 | 0.7 | 0.6 | |
| `cargo_deny` | 4 | 0.2 | 0.3 | |
| `publish_surface` | 8 | 0.7 | 0.7 | |
| `version_consistency` | 2 | 1.1 | 1.2 | |
| `typos` | 1 | 0.2 | 0.2 | |
| `route_ci_runner` | 1 | 0.03 | 0.05 | |
| `mutation_required` | 45 | 2.3 | skipped | Push-only; scoped to changed files |
| `wasm_compile_test` | 25 | 1.6 | skipped | Label-gated on PR |
| `proptest_smoke` | 8 | 0.8 | skipped | Label-gated on PR |

PR Plan static total for default-PR lanes (no changed files): **103 LEM**
(high-cost band). Measured core CI jobs on an ordinary PR: **~16 LEM**.
The gap is inflated static floors plus advisory workflow lanes not in the
aggregate `needs` payload.

## Comparison to older nextest/caching research

ADR-0012 proposed `cargo-nextest` for test execution and separate doctests.
Current gate consolidation already runs `cargo test --all-features` inside
`tokmd_rust_result` on self-hosted (~5–9 min measured). Nextest adoption
would require:

1. Fresh before/after timing on the gate job with and without nextest.
2. Doctest lane separation if nextest excludes them.
3. Self-hosted runner cache state normalization.

**Verdict:** nextest remains a valid future slice but is **not justified** from
current evidence. Measured gate time is dominated by compile+test on self-hosted,
not by test-runner overhead alone. Revisit when perf-smoke or gate-job profiling
shows test collection/execution as the top hotspot.

Similarly, large CI restructuring from pre-consolidation research (150+ LEM
fan-out to Linux+Windows+WASM+mutation on every PR) is **already addressed**
by risk-pack routing and the single tight gate. Current bottleneck is static
floor inflation in PR Plan, not missing parallelization.

## Static floor adjustments (this refresh)

Conservative retuning: `max(measured_p95 × 1.5, 2)` rounded up, keeping
headroom for cold cache on the gate lane.

| Lane | Old floor | New floor | Rationale |
| --- | ---: | ---: | --- |
| `tokmd_rust_result` | 25 | 15 | PR measured 8.8 min; keep cold-cache headroom |
| `feature_boundaries` | 10 | 5 | Stable ~2 min on both runs |
| `docs_check` | 4 | 2 | ~0.8–1.0 min measured |
| `msrv_check` | 5 | 2 | ~0.6–0.7 min measured |
| `publish_surface` | 8 | 2 | ~0.7 min measured |
| `ci_detect_risk_packs` | 1 | 2 | Measured 1.2–1.5 min exceeds old floor |
| `mutation_required` | 45 | 10 | Scoped push run 2.3 min; label-gated on PR |
| `wasm_compile_test` | 25 | 8 | Main push 1.6 min when selected |
| `proptest_smoke` | 8 | 3 | Main push 0.8 min when selected |

Unchanged lanes retain their floors until more samples accumulate.

## Revised default-PR static estimate

After floor retuning, `cargo xtask ci-plan` on `main` with no changed files
reports approximately **78 LEM** (high-cost band, just above elevated) instead
of 103 LEM. Learned estimates will further refine individual lanes as the
`main` actuals cache grows.

## Claim boundary

- **Establishes:** hosted job durations for two recent runs, comparison to
  static floors, and conservative floor retuning rationale.
- **Does not establish:** cross-platform stability, queue-time impact, or
  permission to skip existing proof gates.
- **Does not recommend:** nextest adoption, large CI restructuring, or gate
  promotion changes without fresh profiling evidence.

## See also

- [ci-actuals.md](ci-actuals.md) — receipt contract
- [learned-estimates.md](learned-estimates.md) — planner calibration
- [lem-budgeting.md](lem-budgeting.md) — LEM bands and worked example
- [perf-smoke.md](perf-smoke.md) — local workflow timing (product hot paths)
