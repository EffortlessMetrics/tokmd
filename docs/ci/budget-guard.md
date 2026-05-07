# Soft budget guard

PR 14 wires `cargo xtask ci-plan --enforce` into the PR Plan workflow.
Estimated LEM bands map to advisory severity:

| Estimated LEM | Behavior | Override |
|---------------|----------|----------|
| `0–35` (normal) | green | — |
| `36–75` (elevated) | warning | `ci-budget-ack` suppresses |
| `76–125` (high-cost) | strong warning | `ci-budget-override` or `full-ci` |
| `>125` (override-required) | non-zero exit (when `--enforce`) | `ci-budget-override` or `full-ci` |

The PR Plan workflow runs with `--enforce`. Without an override label, a
plan that estimates over the hard ceiling fails the PR Plan job and
shows a `::error::` annotation explaining what to do (apply the label,
or split the PR).

Bands ≤ hard ceiling never fail; they only emit `::warning::`
annotations and a step-summary banner. This matches the rollout intent:

- Don't fail normal 40 LEM PRs until learned actuals exist (PR 15).
- Visibly nudge for elevated band so reviewers know the PR is broad.
- Refuse the worst case (>125) without an explicit override.

## Static estimates today; learned tomorrow

`base_lem` in `policy/ci-lane-whitelist.toml` is the static floor used
until PR 15 swaps in `p50 × 1.15` learned from `ci-actuals.json`. The
guard logic is the same in both phases; only the input changes.

## Suppressions

- `ci-budget-ack` — apply when the PR is intentionally elevated and the
  reviewer has confirmed the spend is worth it.
- `ci-budget-override` — bypass the hard ceiling for one PR. Use
  sparingly.
- `full-ci` — also acts as override; the deep lanes will run anyway.
