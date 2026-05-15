# PR Plan

The PR Plan job (`.github/workflows/pr-plan.yml`) emits an advisory
`ci-plan.json` for every PR. It is the source of truth for which risk
packs the change touches, which lanes will run, and the estimated LEM
band.

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

8. Write `target/ci/ci-plan.json` and append a Markdown summary to
   `GITHUB_STEP_SUMMARY`.

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

## Status

- **PR 08 (this PR)** — adds the planner, the workflow, and the schema.
  The plan is **advisory**: existing CI still routes via `affected proof
  plan` and the static workflow.
- **CI risk-pack outputs** — `cargo xtask ci-plan --github-output <path>`
  writes workflow-compatible risk-pack booleans for the CI detect job. The
  workflow keeps existing `needs.detect.outputs.*` names, but path
  classification now comes from the Rust-owned planner and checked
  `policy/ci-risk-packs.toml` rather than duplicated shell matching.
- **PR 14** — adds the soft budget guard that warns above the elevated
  limit and fails above the hard limit.
- **PR 15** — replaces static `base_lem` with learned p50/p90/p95
  estimates from `ci-actuals.json` (PR 13).

## Local invocation

```bash
cargo xtask ci-plan \
  --base "origin/main" \
  --head HEAD \
  --labels-json '[{"name":"full-ci"}]' \
  --json-out target/ci/ci-plan.json
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
