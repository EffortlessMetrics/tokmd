# Plan: CI Risk-Pack Outputs

- Status: complete
- Related proposal:
- Related spec:
- Related ADR:
- Related issues:

## Goal

Move PR risk-pack output flags out of workflow-local shell matching and into the
Rust-owned `cargo xtask ci-plan` planner.

The CI workflow should keep acting as a runner, cache, and artifact shell. Path
classification should come from checked policy and xtask code:

```text
policy/ci-risk-packs.toml
  -> cargo xtask ci-plan
  -> ci-plan.json
  -> GitHub output flags
  -> existing risk-gated workflow jobs
```

## Non-goals

- Do not change which CI jobs are required.
- Do not promote advisory proof, scoped coverage, mutation, fast proof, or
  Codecov upload.
- Do not change public `tokmd` CLI behavior or receipt schemas.
- Do not replace `ci/proof.toml` affected-proof routing.
- Do not add a new user-facing command.

## Work Packets

1. Teach `cargo xtask ci-plan` to write GitHub output flags.
   - Status: complete.
   - Add an optional `--github-output <PATH>` flag.
   - Preserve the existing `ci-plan.json` and step-summary behavior.
   - Keep output names compatible with `.github/workflows/ci.yml`.
2. Replace the inline Bash risk-pack classifier in CI.
   - Status: complete.
   - The detect job should call `cargo xtask ci-plan` and consume its
     GitHub-output file.
   - Preserve existing downstream `needs.detect.outputs.*` names.
3. Verify policy coverage.
   - Status: complete.
   - Ensure `policy/ci-risk-packs.toml` covers paths previously hard-coded in
     the workflow detector.
   - Keep affected planning at zero unknown files.

## Decision

Outcome: **complete; risk-pack output classification is Rust-owned**.

The CI detect job still exposes the same downstream output names, but it now
delegates path classification to `cargo xtask ci-plan --github-output`. The
workflow remains a runner/cache/artifact shell: it fetches the base ref,
invokes xtask, and passes through the generated GitHub output flags.

The slice did not change required checks, advisory proof status, Codecov
defaults, public `tokmd` CLI behavior, or public receipt schemas.

## Validation

```bash
cargo test -p xtask ci_plan --verbose
cargo xtask ci-plan --base origin/main --head HEAD --json-out target/ci/ci-plan.json --github-output target/ci/ci-plan.outputs
cargo xtask proof-policy --check
cargo xtask affected --base origin/main --head HEAD --json-output target/proof/affected-ci-risk-pack-outputs.json
cargo xtask proof --profile affected --base origin/main --head HEAD --plan --plan-json target/proof/proof-plan-ci-risk-pack-outputs.json --evidence-json target/proof/proof-evidence-ci-risk-pack-outputs.json
cargo fmt-check
git diff --check
```

If workflow or proof-policy changes select required affected proof, run it and
verify the summary before merging.

## Stop Conditions

- Stop if preserving the existing workflow output names requires changing the
  public `ci-plan.json` schema.
- Stop if the workflow starts making advisory proof required.
- Stop if risk-pack routing cannot preserve existing job-selection behavior.
- Stop if affected planning reports unknown files.
- Stop if generated `target/` artifacts are staged or committed.

## Checkpoint History

- 2026-05-15: Started after the proof-observation decision-readiness lane
  closed with continued observation. The next proof-orchestration gap is the
  CI detect job's inline shell path classifier, which duplicates
  `policy/ci-risk-packs.toml` instead of consuming Rust-owned planner output.
- 2026-05-15: Closed through PR #2281. `cargo xtask ci-plan` now emits
  workflow-compatible output flags, `.github/workflows/ci.yml` consumes those
  flags from the detect job, `policy/ci-risk-packs.toml` covers the previously
  hard-coded release/Nix path cases, and hosted PR plus post-merge CI checks
  passed.
