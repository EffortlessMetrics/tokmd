# PR Plan

The PR Plan job (`.github/workflows/pr-plan.yml`) emits a `ci-plan.json`
artifact for every PR. It is the source of truth for the planner's view of
changed files, hit risk packs, selected lane IDs, and the estimated LEM band.

It is not execution proof. Workflows can still skip conditional jobs, fail
after selection, or be superseded by a newer run for the same head. Open the
route receipt for changed-file routing, the hosted check list for actual job
state, and execution receipts such as `ci-actuals.json` or
`proof-run-summary.json` before treating proof as run.

Lane selection is advisory, but budget enforcement is active: when `--enforce`
estimates more than the hard LEM ceiling, the PR Plan job fails until the PR is
split or an explicit override label is present.

## How it works

1. Fetch the base ref and compute `git diff --name-only base...head`.
2. Read `policy/ci-lane-whitelist.toml` for the lane catalogue + budget.
3. Read `policy/ci-risk-packs.toml` for path -> lane routing.
4. For each non-expensive `default_pr` lane: include it in the advisory plan.
5. For each risk pack whose paths match a changed file: include its
   `lanes`, and (if a matching label or `full-ci` is set) its
   `deep_lanes`.
6. Include any lane whose `policy/ci-lane-whitelist.toml` entry names a
   matching lane-selection label, such as `windows`, `wasm`, `coverage`, or
   `mutation`.
7. Compute the runner-multiplied LEM estimate per lane and the total.
8. Classify the band:

   | Band | LEM range |
   |------|-----------|
   | `normal` | <= default_limit_lem (35) |
   | `elevated` | <= elevated_limit_lem (75) |
   | `high-cost` | <= hard_limit_lem (125) |
   | `override-required` | > hard_limit_lem |

9. Write `target/ci/ci-plan.json`, write
   `target/ci/proof-pack-route.json`, emit budget annotations, and append a
   Markdown summary to `GITHUB_STEP_SUMMARY`.
10. Upload both receipt files as a strict artifact. A missing PR plan or route
   receipt is a workflow failure because reviewers lose the actionable routing
   evidence.

## Output

The route receipt is the first artifact to open when a path did not select the
expected proof. `changed_files` records manifest matches, `unmatched_files`
records paths with no route, and `skipped_by_policy` records lanes that were
known but intentionally not selected. Route-relevant unselected lanes are
reported even when they are not expensive, so deep proof packs can be audited
without reading the lane catalogue by hand:

Each `changed_files[]` row names the changed path in `changed_file`, the
canonical surface in `surface`, and the proof packs required by that surface in
`required_packs`. The older `path` and `proof_packs` fields remain in the same
row as compatibility aliases for existing cockpit and handoff readers.

Specific authority packs can supersede generic packs in
`policy/ci-risk-packs.toml`. For example, handoff and review-packet contract
docs route as `handoff_review_packet` instead of also appearing as generic
`docs`, so the receipt names the surface that actually owns the proof.

```json
{
  "schema": "tokmd.proof_pack_route.v1",
  "schema_version": 5,
  "changed_files": [
    {
      "changed_file": "crates/tokmd/src/main.rs",
      "path": "crates/tokmd/src/main.rs",
      "surface": "core_receipts",
      "required_packs": ["core_receipts"],
      "proof_packs": ["core_receipts"],
      "reason": "manifest_match",
      "policy": "blocking",
      "lanes": ["build_test_linux", "proof_policy", "ripr_advisory"],
      "deep_lanes": ["build_test_windows", "proptest_smoke"]
    }
  ],
  "unmatched_files": [],
  "summary": {
    "changed_file_count": 1,
    "routed_file_count": 1,
    "unmatched_file_count": 0,
    "skipped_lane_count": 2,
    "skipped_reason_counts": {
      "deep_lane_requires_label": 2
    }
  },
  "skipped_by_policy": [
    {
      "lane": "build_test_windows",
      "status": "skipped_by_policy",
      "reason": "deep_lane_requires_label",
      "matched_files": ["crates/tokmd/src/main.rs"],
      "lane_kind": "rust",
      "tier": "risk-gated",
      "blocking": true,
      "expensive": true,
      "required_labels": ["windows"],
      "estimated_lem": 40,
      "estimate_source": "static"
    },
    {
      "lane": "proptest_smoke",
      "status": "skipped_by_policy",
      "reason": "deep_lane_requires_label",
      "matched_files": ["crates/tokmd/src/main.rs"],
      "lane_kind": "property",
      "tier": "risk-gated",
      "blocking": true,
      "expensive": false,
      "required_labels": ["property-tests"],
      "estimated_lem": 8,
      "estimate_source": "static"
    }
  ]
}
```

The summary reason counts are an at-a-glance index over the detailed
`skipped_by_policy` array. Use the per-lane entries for matched files and
lane-specific evidence. Route receipt v5 also records `lane_kind`, `tier`,
`blocking`, `expensive`, `required_labels`, `estimated_lem`, `estimate_source`,
and optional learned percentile fields for each skipped row so skipped deep
proof can be audited without opening the lane whitelist. The estimate fields
explain policy cost; they do not mean the skipped lane ran, passed, or seeded a
learned estimate sample:

- `deep_lane_requires_label` means a matching surface has label-gated deep proof
  that was not requested.
- `not_selected_by_policy` means the lane matched the changed surface directly,
  but the current planner policy did not select it for this run.
- `docs_only_change` means an expensive lane was skipped because all routed
  files were documentation.
- `not_selected_for_changed_surface` means the lane is known to the planner but
  does not apply to the current changed surface.
- `no_changed_files` means the planner saw no changed paths for the comparison
  range, so expensive lanes were reported as skipped rather than silently
  omitted.

The advisory plan keeps its existing shape. Treat `lanes_selected` as planner
selection, not proof that those jobs executed or passed:

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
      "id": "build_test_linux",
      "workflow": ".github/workflows/ci.yml",
      "job": "Build & Test (Linux)",
      "kind": "rust",
      "tier": "frontdoor",
      "runner": "ubuntu_latest",
      "blocking": true,
      "estimated_lem": 12,
      "estimate_source": "static",
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
- `cargo xtask ci-plan --route-json-out <path>` writes the changed-file route
  receipt used to see matched proof packs, unmatched files, and explicit
  skipped-by-policy lanes before broad CI proof starts.
- The GitHub step summary shows each selected lane's estimate source so
  reviewers can tell static floors from learned actuals without opening
  `ci-plan.json`.
- `--enforce` fails only the hard-ceiling band (`override-required`) when
  neither `ci-budget-override` nor `full-ci` is present. Lower bands emit
  warnings but do not fail the job.
- `ci-budget-ack` acknowledges elevated and high-cost warnings below the hard
  ceiling. It does not bypass `override-required`.
- The hosted PR Plan workflow downloads recent successful `CI` run
  `ci-actuals` artifacts from `main` into `target/ci/actuals-cache` when
  GitHub makes them available, then passes that directory to `ci-plan` with
  `--actuals-dir`. The download is advisory: if no receipt is available or the
  lookup fails, the planner falls back to static `base_lem` values from
  `policy/ci-lane-whitelist.toml`.

## CI Actuals Interpretation

`target/ci/ci-actuals.json` is written by the advisory `CI Actuals (Advisory)`
job from the same run's `needs` payload. Open it when you need the observed
required-job results and timing coverage behind the required `Tokmd Rust Result`
check.

Treat the fields as telemetry, not a replacement verdict:

- `status.ok` means the receipt was generated successfully. It does not mean
  every CI job passed.
- `jobs[].result` is the per-required-job result from GitHub Actions `needs`.
  Use it to find failed, cancelled, skipped, or successful inputs to the
  aggregate.
- `jobs[].summary_key` is the aggregate `needs` key, while `jobs[].lane_id`
  and `jobs[].aliases` make the row usable by canonical lane-id planning.
- `jobs[].selected` records whether the workflow selected the job for
  execution. Skipped rows include `skip_reason` when the workflow exposed one,
  otherwise `github_actions_condition_false`.
- `route_target`, `estimated_lem`, `actual_lem`, and `queue_seconds` are
  nullable telemetry fields. Missing values mean the workflow did not observe
  them.
- `estimate_source` is nullable telemetry copied from aggregate job outputs
  when available. Use it to distinguish static LEM floors from advisory learned
  estimate sources in both the workflow summary and `ci-actuals.json`.
- `status.missing_timing` means timing telemetry was unavailable for those
  jobs. It is not a zero-second duration and not a job failure by itself.
- `duration_seconds`, `duration_minutes`, `runner`, and `cache_hit` are cost
  observations. They do not promote learned estimates, change required gates,
  or make a skipped job passing evidence.
- `status.unused_timing` records timing sidecar entries that did not match a
  required job key, so routing or timing-name drift can be spotted without
  treating the receipt as stronger proof.

The aggregate CI job also appends a `CI Actuals (advisory)` table to the
workflow summary. Use it for first-read diagnosis of selected lanes, expected
and actual LEM, duration, route target, learned-estimate source, and skip
reasons. Download `ci-actuals.json` when you need the stable machine-readable
receipt.

## Runner routing (Route CI runner)

The separate routed Rust Small frontdoor was retired in phase 3 (#299). Runner
routing for `tokmd-swarm` workbench PRs now lives in the advisory `Route CI
runner` job of `ci.yml`: it selects a trusted self-hosted runner when idle
healthy capacity exists and falls back to GitHub-hosted overflow otherwise. The
single required `Tokmd Rust Result` gate then runs on the selected runner.

There are no longer separate mutually-exclusive implementation jobs or an
aggregate `Tokmd Rust Small Result` lane to budget. PR Plan budgets the gate
through the `tokmd_rust_result` lane; the advisory `Route CI runner` job is
cheap and never required. Runner health and fallback behavior are documented in
`docs/ci/runner-health-runbook.md`; the historical routing contract is preserved
in `docs/ci/routed-ci-policy.md`.

## Override handling

Use `ci-budget-ack` when a PR is intentionally elevated or high-cost but still
below the hard ceiling. Use `ci-budget-override` when a high-LEM PR is
intentionally broad enough to exceed the hard ceiling and splitting would make
the evidence worse. Use `full-ci` when the PR should also request every default
blocking lane.

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
  --route-json-out target/ci/proof-pack-route.json \
  --enforce
```

To generate the same output flags consumed by `.github/workflows/ci.yml`:

```bash
cargo xtask ci-plan \
  --base "origin/main" \
  --head HEAD \
  --labels-json '[{"name":"full-ci"}]' \
  --json-out target/ci/ci-plan.json \
  --route-json-out target/ci/proof-pack-route.json \
  --github-output target/ci/ci-plan.outputs
```
