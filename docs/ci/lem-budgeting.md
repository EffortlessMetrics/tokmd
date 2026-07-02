# LEM: Lane-Equivalent Minutes

`LEM` is the operating unit we use to compare CI cost across runners and
lanes.

```text
LEM = wall-clock job minutes × runner multiplier
```

The runner multiplier normalizes runner pricing to `ubuntu-latest = 1.0`.

## Default multipliers

| Runner | Multiplier | Reasoning |
|--------|------------|-----------|
| `ubuntu-latest` | 1.0 | Baseline. |
| `windows-latest` | 2.0 | GitHub-hosted Windows minutes are billed at 2× Linux. |
| `macos-latest` | 10.0 | GitHub-hosted macOS minutes are billed at 10× Linux. |
| `nix-build` | 4.0 | Nix evaluator + sandbox cost dominates wall-clock. |
| `external-ai-review` | 4.0 | LLM-bound lane, rate-limit-bound, capped budget. |

The canonical multipliers live in `policy/ci-lane-whitelist.toml` under
`[runner_multipliers]`.

## Bands

| Band | LEM | Meaning |
|------|-----|---------|
| Pennies | 0–12 | Tiny PR, docs-only, single-crate change. |
| Normal | 13–35 | Default sub-$0.50 ordinary PR target. |
| Elevated | 36–75 | Risk-pack-hit PR. Warns. |
| High-cost | 76–125 | Known-broad or heavily routed change. Strong warning; `ci-budget-ack` may acknowledge. |
| Override | >125 | Requires `full-ci` or `ci-budget-override`. |

## Worked example

A typical Rust-only PR after gate consolidation (#226 phase 2, #299 phase 3).
Quality Gate, Proof Policy, and Build & Test (Linux) are folded into the single
required `tokmd_rust_result` gate:

```text
PR Plan (advisory)            1 LEM
Route CI runner               1 LEM
Tokmd Rust Result            15 LEM   (gate + test + proof-policy concurrent)
Affected Proof Plan           4 LEM
Detect risk packs             2 LEM
Feature Boundaries            5 LEM
MSRV Check                    2 LEM
Docs Check                    2 LEM
Cargo Deny                    4 LEM
Publish Surface               2 LEM
Version consistency           2 LEM
Typos                         1 LEM
ripr (advisory)               2 LEM
PR Cockpit (advisory)         3 LEM
No-panic / Clippy policy      6 LEM
CI policy lanes               5 LEM
Fast proof run (advisory)     5 LEM
Scoped coverage (advisory)   12 LEM
                            ------
                             78 LEM  (high-cost band; measured core CI ~16 LEM)
```

Risk-gated lanes (Windows, WASM, Nix, mutation, proptest) add cost only when
labels or `full-ci` select them. Pre-consolidation research cited 150+ LEM for
every-PR fan-out; that path is retired. See
[timing-refresh-2026-07.md](timing-refresh-2026-07.md) for measured evidence.

## Estimation vs. actuals

By default, estimates are **static floors** taken from
`policy/ci-lane-whitelist.toml :: base_lem`. When a caller provides
`--actuals-dir` with past `ci-actuals.json` receipts, the planner uses:

```text
estimate     = max(static_floor, p50_recent_actual × 1.15)
warning      = p90_recent_actual
hard ceiling = p95_recent_actual
```

The planner treats the uploaded CI aggregate `needs` keys as telemetry names,
not lane ids. It normalizes hyphenated keys such as `docs-check` and maps
known aggregate names such as `build`, `msrv`, `mutation`, and `nix-pr` to
their lane ids before using the samples.

The static floor still applies in learned mode so a brand-new lane never
reports `0 LEM` because no data has been collected yet. The hosted PR Plan
workflow uses a best-effort cache of recent successful `main` CI actuals and
falls back to static estimates when no valid cache is available.
