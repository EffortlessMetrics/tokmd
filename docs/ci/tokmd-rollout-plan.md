# tokmd CI rollout plan

This file pins the rollout sequence so reviewers can see where any given PR
fits in the ladder. Every PR in this stack has the same shape:

- **Default PR LEM impact** — what the band looks like before/after.
- **Workflows touched** — which `.yml` files change.
- **Branch protection impact** — what required jobs change, if any.
- **Failure mode caught** — what proof we are buying with the change.
- **Cheaper signal considered** — what we considered and rejected.
- **Rollback path** — how to revert without losing the receipt model.
- **Commands run** — exact local verification commands.

## Stack

| PR | Branch                           | Purpose                                                                  |
| -: | -------------------------------- | ------------------------------------------------------------------------ |
| 01 | `ci/verification-economics-docs` | Verification economics + LEM + contributor docs.                         |
| 02 | `ci/lane-whitelist`              | Map every CI item to purpose, cost, trigger, owner, evidence.            |
| 03 | `ci/lane-whitelist-lint`         | `xtask` check so workflows cannot drift from lane policy.                |
| 04 | `policy/msrv-1-93-ledger`        | Move MSRV to 1.93 and ledger future Clippy flips.                        |
| 05 | `policy/non-rust-allowlist`      | TOML allowlist of non-Rust surfaces.                                     |
| 06 | `policy/no-panic-allowlist`      | Semantic no-panic allowlist (extends existing TOML).                     |
| 07 | `policy/clippy-exceptions`       | AST-backed Clippy exception ledger.                                      |
| 08 | `ci/pr-plan-advisory`            | LEM-aware `ci-plan.json` wrapping the proof plan.                        |
| 09 | `ci/cache-and-cancel-policy`     | save-only-main caches; label-event-safe cancellation.                    |
| 10 | `ci/default-pr-gate-slimming`    | Make default PR gate cheap.                                              |
| 11 | `ci/mutation-to-ripr-default`    | Move runtime mutation off default PR; add ripr advisory.                 |
| 12 | `ci/risk-pack-routing`           | Route WASM, Windows, Nix, publish, proptest by risk pack.                |
| 13 | `ci/nextest-junit-actuals`       | JUnit/nextest timing and `ci-actuals.json`.                              |
| 14 | `ci/soft-budget-guard`           | Warn >35/>75 LEM; fail >125 without override.                            |
| 15 | `ci/learned-estimates`           | Use actuals for rolling p50/p90/p95 LEM estimates.                       |
| 16 | `ripr/soft-gate-policy`          | Acknowledge high-confidence new oracle gaps.                             |

## Non-goals

- Replacing the proof model. The receipts must still be the proof.
- Removing strict-lint posture. Clippy strictness stays.
- Removing mutation testing. It moves to calibration / nightly / labels.
- Hard-enforcing learned LEM budgets before `ci-actuals` exist.
