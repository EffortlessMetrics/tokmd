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
| High-cost | 76–125 | Explicit label or known-broad change. Strong warning. |
| Override | >125 | Requires `full-ci` or `ci-budget-override`. |

## Worked example

A typical Rust-only PR that hits the `core_receipts` risk pack:

```text
PR Plan                       1 LEM
Quality Gate                  8 LEM
Proof Policy                  3 LEM
Affected Proof Plan           4 LEM
Scoped Rust fast gate        12 LEM
ripr advisory                 2 LEM
Typos                         1 LEM
CI Required summary           1 LEM
                            ------
                             32 LEM  (Linux only, normal band)
```

Compare to the same change today, which fans out to Linux + Windows
all-features test, WASM compile + Node tests, Nix package gate, and runtime
mutation testing — easily 150+ LEM before risk-pack routing.

## Estimation vs. actuals

Until `ci-actuals.json` calibration data exists, estimates are **static
floors** taken from `policy/ci-lane-whitelist.toml :: base_lem`. Once
actuals are collected (PR 13), the planner uses:

```text
estimate     = max(static_floor, p50_recent_actual × 1.15)
warning      = p90_recent_actual
hard ceiling = p95_recent_actual
```

The static floor exists so a brand-new lane never reports `0 LEM` because
no data has been collected yet.
