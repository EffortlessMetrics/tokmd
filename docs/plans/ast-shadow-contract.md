# Plan: AST Shadow Contract

- Status: complete
- Related proposal:
- Related spec: `docs/specs/ast-shadow.md`
- Related ADR: `docs/adr/0008-ast-foundation.md`
- Related issues:

## Goal

Define the first focused AST shadow artifact contract without changing product
behavior.

The plan should make the next AST implementation slice safer by recording:

```text
what the shadow artifacts are
what they are not
which default outputs must remain unchanged
which proof scope owns the contract
when a future schema review is required
```

## Non-goals

- Do not emit new AST artifacts in this PR.
- Do not change default `tokmd` output, schemas, browser capabilities, bindings,
  or CI behavior.
- Do not promote proof gates, scoped coverage, mutation, or Codecov upload.
- Do not add an evidencebus runtime dependency.
- Do not add a public `tokmd ast` command.
- Do not make AST the default source for analysis receipts.

## Work Packets

1. Add `docs/specs/ast-shadow.md`.
   - Status: complete.
   - Define inputs, outputs, compatibility boundaries, and proof requirements
     for `tokmd.ast_shadow.v1`.
2. Link ADR-0008 to the focused spec.
   - Status: complete.
   - Keep ADR-0008 as the durable architecture decision and the spec as the
     behavior/proof contract.
3. Route the spec through proof policy.
   - Status: complete.
   - Add the spec path to the existing `analysis_ast_shadow` scope so AST
     contract edits select the AST feature tests.
4. Record the lane checkpoint.
   - Status: complete.
   - Update `docs/NEXT.md` and `.jules/goals/active.toml` without reopening the
     completed product-readiness or doc-artifacts lanes.

## Validation

```bash
cargo xtask doc-artifacts --check
cargo xtask docs --check
cargo test -p tokmd-analysis --features ast ast --verbose
cargo xtask proof-policy --check
cargo xtask affected --base origin/main --head HEAD --json-output target/proof/affected-ast-shadow.json
cargo xtask proof --profile affected --base origin/main --head HEAD --plan --plan-json target/proof/proof-plan-ast-shadow.json --evidence-json target/proof/proof-evidence-ast-shadow.json
cargo fmt-check
git diff --check
```

## Stop Conditions

- Stop if the spec requires behavior that is not implemented or deliberately
  planned as future work.
- Stop if affected planning reports unknown files.
- Stop if the contract implies AST-backed default receipt behavior.
- Stop if the spec claims browser/WASM AST capability without capability-matrix
  and bundle-size evidence.

## Checkpoint History

- 2026-05-14: Added the focused AST shadow artifact contract as a docs-only
  groundwork slice after the first product-readiness pass and artifact glossary
  lane completed.
