# Plan: Mutation Summary Parsing

- Status: complete
- Related proposal:
- Related spec:
- Related ADR:
- Related issues:

## Goal

Move the manual mutation workflow's summary and survivor parsing out of inline
workflow shell and into Rust-owned `xtask` code while preserving the existing
`mutants-summary.json` artifact shape.

The workflow should remain a runner, cache, and artifact shell:

```text
cargo-mutants execution
  -> copied mutants.out directories
  -> cargo xtask mutation-summary
  -> mutants-summary.json + GitHub output flags
```

This makes mutation result summarization testable and deterministic without
changing whether mutation is advisory, required, or product-visible.

## Non-goals

- Do not promote mutation testing into a required aggregate gate.
- Do not change Codecov upload behavior.
- Do not change public `tokmd` CLI behavior or public receipt schemas.
- Do not change how the workflow invokes `cargo-mutants`.
- Do not replace `cargo xtask proof --plan` mutation planning.
- Do not make mutation summary output a cockpit, handoff, or merge verdict.

## Work Packets

1. Add Rust-owned mutation summary parsing.
   - Status: complete.
   - Add `cargo xtask mutation-summary`.
   - Preserve the current `mutants-summary.json` fields:
     `schema_version`, `commit`, `base_ref`, `status`, `scope`, `survivors`,
     `killed`, `timeout`, and `unviable`.
   - Preserve workflow-compatible `status` and `survivor_count` GitHub outputs.
   - Parse `outcomes.json` when present and text fallbacks when it is missing.
2. Wire the manual mutation workflow.
   - Status: complete.
   - Keep `cargo-mutants` execution behavior unchanged.
   - Keep artifact names and upload behavior unchanged.
3. Checkpoint the remaining mutation workflow shell.
   - Status: complete.
   - Mutation execution orchestration stays workflow-owned unless a future plan
     identifies a concrete consumer or maintenance problem.

## Decision

Outcome: **complete; mutation summary and survivor parsing are Rust-owned**.

PR #2294 added `cargo xtask mutation-summary`, preserving the manual mutation
workflow's existing `mutants-summary.json` fields:

```text
schema_version
commit
base_ref
status
scope
survivors
killed
timeout
unviable
```

The command also preserves the workflow-compatible `status` and
`survivor_count` GitHub outputs. The workflow still invokes `cargo-mutants` the
same way and still uploads the same artifact family; it now asks `xtask` to
parse copied `mutants.out` directories and text fallbacks.

The slice did not promote mutation testing, change Codecov behavior, change
public `tokmd` CLI behavior, change public receipt schemas, replace
`cargo xtask proof --plan` mutation planning, or make mutation summary output a
cockpit, handoff, or merge verdict.

## Validation

```bash
cargo test -p xtask mutation_summary --verbose
cargo xtask mutation-summary --commit HEAD --base-ref main --scope-exceeded false --mutants-ran false --json-output target/mutation/mutants-summary.json --github-output target/mutation/mutation-summary.outputs
cargo xtask proof-policy --check
cargo xtask doc-artifacts --check
cargo xtask docs --check
cargo xtask affected --base origin/main --head HEAD --json-output target/proof/affected-mutation-summary-parsing.json
cargo xtask proof --profile affected --base origin/main --head HEAD --plan --plan-json target/proof/proof-plan-mutation-summary-parsing.json --evidence-json target/proof/proof-evidence-mutation-summary-parsing.json
cargo fmt-check
git diff --check
```

Run required affected proof if the affected plan selects it.

## Stop Conditions

- Stop if preserving existing `mutants-summary.json` requires changing summary
  semantics.
- Stop if the workflow starts making mutation required.
- Stop if the new summary task needs a public `tokmd` schema or CLI surface.
- Stop if affected planning reports unknown files.
- Stop if generated `target/` artifacts are staged or committed.

## Checkpoint History

- 2026-05-15: Started after mutation scope selection closed and a fresh
  workflow audit found `.github/workflows/mutants.yml` still owns summary and
  survivor parsing through inline Bash/JQ. This slice moves parsing only and
  leaves mutation execution orchestration unchanged.
- 2026-05-15: Closed through PR #2294. Hosted PR checks passed, post-merge main
  CI passed, and the manual mutation workflow now uses
  `cargo xtask mutation-summary` while preserving advisory mutation behavior.
