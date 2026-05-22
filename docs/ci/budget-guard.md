# Soft budget guard

The PR Plan workflow runs `cargo xtask ci-plan --enforce`. Estimated LEM bands
map to advisory severity:

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

- Don't fail elevated-but-normal PRs while estimate calibration is still coarse.
- Visibly nudge for elevated band so reviewers know the PR is broad.
- Refuse the worst case (>125) without an explicit override.

## Static and learned estimates

`base_lem` in `policy/ci-lane-whitelist.toml` is the static floor. The hosted
PR Plan workflow uses this static floor unless a future workflow change provides
an `--actuals-dir` cache of past `ci-actuals.json` receipts.

When `--actuals-dir` is provided, `cargo xtask ci-plan` can estimate lanes with
`max(static_floor, p50_recent_actual × 1.15)` while still reporting p90 and p95
context. The guard logic is the same in both modes; only the estimate input
changes.

## Suppressions

- `ci-budget-ack` — apply when the PR is intentionally elevated and the
  reviewer has confirmed the spend is worth it.
- `ci-budget-override` — bypass the hard ceiling for one PR. Use
  sparingly. This is appropriate when the PR is intentionally broad, or when a
  narrow PR has a known static-estimator overcount such as mutually exclusive
  routed Rust Small implementation lanes. In the overcount case, cite the
  affected proof plan, latest passing PR Plan run for the current head SHA, and
  hosted check rollup.
- `full-ci` — also acts as override; the deep lanes will run anyway.

If a label is added after an `override-required` failure, use the latest PR Plan
run for the same head SHA after the label is visible. Rerunning the failed job
is acceptable; older failed attempts may remain in the check history, but they
are not the current budget signal once a newer run passes with the override.
