# Plan: CI Mutation Scope Routing

- Status: active
- Related proposal:
- Related spec:
- Related ADR:
- Related issues:

## Goal

Make the CI mutation job use the existing Rust-owned mutation scope selector
instead of maintaining a second inline changed-file classifier in workflow
shell.

Today the manual mutation workflow already uses:

```text
cargo xtask mutation-scope
  -> changed_files.txt
  -> target/mutation/mutation-scope.json
  -> workflow-compatible outputs
```

The CI mutation job in `.github/workflows/ci.yml` still computes changed Rust
files with `git diff | grep ...` before invoking `cargo-mutants`. That duplicate
classifier can drift from the tested `xtask` selector. This slice should route
CI mutation selection through `cargo xtask mutation-scope` while preserving the
existing mutation execution loop.

## Non-goals

- Do not promote mutation testing into a required aggregate gate.
- Do not change Codecov upload behavior.
- Do not change public `tokmd` CLI behavior or public receipt schemas.
- Do not replace `cargo xtask proof --plan` mutation planning.
- Do not rewrite mutation execution, crate-directory selection, or
  survivor-summary parsing.
- Do not make mutation scope output a cockpit, handoff, or merge verdict.
- Do not broaden draft PR #2299 into this lane.

## Work Packets

1. Replace the CI mutation changed-file classifier.
   - Status: pending.
   - In `.github/workflows/ci.yml`, update the mutation job's
     `Get changed Rust files` step to call `cargo xtask mutation-scope`.
   - Preserve `steps.changed.outputs.count` and `steps.changed.outputs.files`
     so the existing execution and skip steps keep their contract.
   - Preserve `changed_files.txt` as the file consumed by the existing
     `cargo-mutants` loop.
2. Emit CI mutation scope evidence.
   - Status: pending.
   - Write `target/mutation/mutation-scope.json` from the CI mutation job, using
     the same `tokmd.mutation_scope.v1` shape as the manual mutation workflow.
   - Upload it with the existing mutation artifacts if practical; if upload
     behavior stays unchanged, document why.
3. Validate policy and affected routing.
   - Status: pending.
   - Ensure `.github/workflows/ci.yml` still routes through the proof-control
     scope and produces zero unknown files.
   - Keep mutation advisory and Codecov default-off.

## Validation

```bash
cargo test -p xtask mutation_scope --verbose
cargo xtask mutation-scope --base origin/main --head HEAD --json-output target/mutation/mutation-scope.json --github-output target/mutation/mutation-scope.outputs
cargo xtask doc-artifacts --check
cargo xtask docs --check
cargo xtask proof-policy --check
cargo xtask affected --base origin/main --head HEAD --json-output target/proof/affected-ci-mutation-scope-routing.json
cargo xtask proof --profile affected --base origin/main --head HEAD --plan --plan-json target/proof/proof-plan-ci-mutation-scope-routing.json --evidence-json target/proof/proof-evidence-ci-mutation-scope-routing.json
cargo fmt-check
git diff --check
```

Run required affected proof selected by the affected plan. Do not run
`cargo-mutants` locally from this slice unless a focused workflow reproduction
specifically requires it.

## Stop Conditions

- Stop if preserving existing `count` / `files` outputs requires changing
  mutation-scope semantics.
- Stop if the workflow would start making mutation required.
- Stop if the workflow needs a public `tokmd` schema or CLI surface.
- Stop if affected planning reports unknown files.
- Stop if generated `target/`, `changed_files.txt`, or
  `all_changed_files.txt` artifacts are staged or committed.

## Checkpoint History

- 2026-05-15: Selected by the proof-orchestration gap audit. The audit found
  `.github/workflows/ci.yml` still owns a duplicate inline changed-file
  classifier for the label/push mutation job, while the manual mutation
  workflow already uses the Rust-owned `cargo xtask mutation-scope` selector.
