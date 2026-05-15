# Plan: Mutation Scope Selection

- Status: active
- Related proposal:
- Related spec:
- Related ADR:
- Related issues:

## Goal

Move the mutation workflow's changed-file scope selection out of inline
workflow shell and into Rust-owned `xtask` code.

The manual mutation workflow should keep acting as a runner, cache, and
artifact shell:

```text
git diff base...head
  -> cargo xtask mutation-scope
  -> mutation-scope.json + GitHub output flags
  -> existing cargo-mutants execution and summary behavior
```

This makes the selection contract testable and deterministic without changing
whether mutation is advisory, required, or product-visible.

## Non-goals

- Do not promote mutation testing into a required aggregate gate.
- Do not change Codecov upload behavior.
- Do not change public `tokmd` CLI behavior or public receipt schemas.
- Do not replace `cargo xtask proof --plan` mutation planning.
- Do not rewrite mutation execution or survivor-summary parsing in this slice.
- Do not make mutation scope output a cockpit, handoff, or merge verdict.

## Work Packets

1. Add Rust-owned mutation scope selection.
   - Status: active.
   - Add `cargo xtask mutation-scope`.
   - Preserve the current production Rust file filters from
     `.github/workflows/mutants.yml`.
   - Emit workflow-compatible `base_ref`, `total_count`, `scope_exceeded`,
     `count`, and `files` outputs.
   - Write deterministic `tokmd.mutation_scope.v1` JSON when requested.
2. Wire the manual mutation workflow.
   - Status: active.
   - Keep `cargo-mutants` execution behavior unchanged.
   - Upload `mutation-scope.json` beside the existing mutation summary.
3. Checkpoint the remaining mutation workflow shell.
   - Status: pending.
   - Decide from fresh evidence whether survivor-summary parsing should move
     into `xtask` later.

## Validation

```bash
cargo test -p xtask mutation_scope --verbose
cargo xtask mutation-scope --base origin/main --head HEAD --json-output target/mutation/mutation-scope.json --github-output target/mutation/mutation-scope.outputs
cargo xtask proof-policy --check
cargo xtask doc-artifacts --check
cargo xtask docs --check
cargo xtask affected --base origin/main --head HEAD --json-output target/proof/affected-mutation-scope-selection.json
cargo xtask proof --profile affected --base origin/main --head HEAD --plan --plan-json target/proof/proof-plan-mutation-scope-selection.json --evidence-json target/proof/proof-evidence-mutation-scope-selection.json
cargo fmt-check
git diff --check
```

Run required affected proof if the affected plan selects it.

## Stop Conditions

- Stop if preserving existing workflow outputs requires changing mutation
  summary semantics.
- Stop if the workflow starts making mutation required.
- Stop if the new scope receipt needs a public `tokmd` schema or CLI surface.
- Stop if affected planning reports unknown files.
- Stop if generated `target/` artifacts are staged or committed.

## Checkpoint History

- 2026-05-15: Started after publishing evidence readiness closed. Fresh
  workflow audit found `.github/workflows/mutants.yml` still owns changed-file
  selection in Bash even though proof planning already records mutation as
  advisory evidence. The first slice makes selection Rust-owned and leaves
  mutation execution plus summary parsing unchanged.
