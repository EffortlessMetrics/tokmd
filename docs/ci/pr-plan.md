# PR Plan

The PR Plan job (`.github/workflows/pr-plan.yml`) emits a `ci-plan.json`
artifact for every PR. It is the source of truth for which risk packs the
change touches, which lanes will run, and the estimated LEM band.

The plan is advisory for lane selection, but budget enforcement is active:
when `--enforce` estimates more than the hard LEM ceiling, the PR Plan job
fails until the PR is split or an explicit override label is present.

## How it works

1. Fetch the base ref and compute `git diff --name-only base...head`.
2. Read `policy/ci-lane-whitelist.toml` for the lane catalogue + budget.
3. Read `policy/ci-risk-packs.toml` for path → lane routing.
4. For each non-expensive `default_pr` lane: include it always.
5. For each risk pack whose paths match a changed file: include its
   `lanes`, and (if a matching label or `full-ci` is set) its
   `deep_lanes`.
6. Compute the runner-multiplied LEM estimate per lane and the total.
7. Classify the band:

   | Band | LEM range |
   |------|-----------|
   | `normal` | ≤ default_limit_lem (35) |
   | `elevated` | ≤ elevated_limit_lem (75) |
   | `high-cost` | ≤ hard_limit_lem (125) |
   | `override-required` | > hard_limit_lem |

8. Write `target/ci/ci-plan.json`, emit budget annotations, and append a
   Markdown summary to `GITHUB_STEP_SUMMARY`.

## Output

```json
{
  "schema_version": 1,
  "base": "origin/main",
  "head": "HEAD",
  "labels": ["wasm"],
  "changed_files": [...],
  "risk_packs_hit": [
    { "name": "wasm", "description": "...", "matched_files": [...] }
  ],
  "lanes_selected": [
    {
      "id": "rust_fast_gate",
      "workflow": ".github/workflows/ci.yml",
      "job": "Rust Fast Gate",
      "kind": "rust",
      "tier": "frontdoor",
      "runner": "ubuntu_latest",
      "blocking": true,
      "estimated_lem": 12,
      "reason": "default_pr"
    }
  ],
  "estimated_lem": 32,
  "band": "normal",
  "budget": { ... }
}
```

## Current behavior

- `cargo xtask ci-plan --github-output <path>` writes workflow-compatible
  risk-pack booleans for the CI detect job. The workflow keeps existing
  `needs.detect.outputs.*` names, but path classification comes from the
  Rust-owned planner and checked `policy/ci-risk-packs.toml` rather than
  duplicated shell matching.
- `--enforce` fails only the hard-ceiling band (`override-required`) when
  neither `ci-budget-override` nor `full-ci` is present. Lower bands emit
  warnings but do not fail the job.
- The hosted PR Plan workflow currently uses static `base_lem` values from
  `policy/ci-lane-whitelist.toml`. `cargo xtask ci-plan` can consume learned
  actuals with `--actuals-dir`, but the workflow must wire that directory in
  before hosted PRs use learned estimates.

## Routed Rust Small Interpretation

`tokmd-swarm` has an additional routed Rust Small frontdoor. The lane catalogue
lists the router, the aggregate `Tokmd Rust Small Result`, and each conditional
implementation job so reviewers can see the whole route surface.

Those implementation jobs are mutually exclusive at runtime. A trusted
same-repo swarm PR normally selects one self-hosted target and skips the other
implementation jobs. A publication-repo PR skips the routed workflow entirely
because it is guarded to `github.repository == 'EffortlessMetrics/tokmd-swarm'`.

PR Plan budgets the routed frontdoor through the router plus the aggregate
`Tokmd Rust Small Result` lane. The conditional implementation lanes stay in
the whitelist inventory but are not ordinary default PR lanes, because only
one implementation can run for a given route. The aggregate result lane uses a
conservative static estimate for one selected implementation path so small
swarm PRs do not require `ci-budget-override` merely because skipped route
targets were counted.

## Override handling

Use `ci-budget-override` when a high-LEM PR is intentionally broad enough to
exceed the hard ceiling and splitting would make the evidence worse. Use
`full-ci` when the PR should also request every default blocking lane.

The workflow runs on `labeled` and `unlabeled` events. If the first PR Plan run
fails before the override label is visible, rerun the failed PR Plan run or wait
for the label-triggered replacement run. Treat the latest successful PR Plan
for the current head SHA as the actionable budget signal; older failed attempts
can remain in the check history.

## Local invocation

```bash
cargo xtask ci-plan \
  --base "origin/main" \
  --head HEAD \
  --labels-json '[{"name":"full-ci"}]' \
  --json-out target/ci/ci-plan.json \
  --enforce
```

To generate the same output flags consumed by `.github/workflows/ci.yml`:

```bash
cargo xtask ci-plan \
  --base "origin/main" \
  --head HEAD \
  --labels-json '[{"name":"full-ci"}]' \
  --json-out target/ci/ci-plan.json \
  --github-output target/ci/ci-plan.outputs
```
