# Plan: AST Rust Landmark Coverage

- Status: complete
- Related proposal:
- Related spec: `docs/specs/ast-shadow.md`
- Related ADR: `docs/adr/0008-ast-foundation.md`
- Related issues:

## Goal

Extend the feature-gated Rust AST shadow parser from function-only landmarks to
the first broader code-intelligence landmark set: imports plus simple
control-flow nodes.

This keeps AST work useful for later heuristic-vs-AST comparison without
changing default receipts, CLI behavior, browser capabilities, proof promotion,
or Codecov defaults.

## Non-goals

- Do not add a public AST command or default runner.
- Do not change `tokmd analyze`, `tokmd cockpit`, `tokmd context`, or
  `tokmd handoff` output.
- Do not add new public receipt fields or schema behavior.
- Do not claim browser/WASM AST support.
- Do not infer semantic import resolution, call graphs, or full control-flow
  graphs.
- Do not duplicate `mergecode`'s deeper semantic graph lane.

## Work Packets

1. Parse Rust import landmarks.
   - Status: complete.
   - Capture `use_declaration` nodes with normalized source text.
2. Parse simple Rust control-flow landmarks.
   - Status: complete.
   - Capture `if`, `match`, `for`, `while`, and `loop` expression nodes as
     deterministic landmarks.
3. Preserve shadow artifact compatibility.
   - Status: complete.
   - Map new AST landmark kinds into the existing `tokmd.ast_shadow.v1`
     artifact builder without changing default receipts.
4. Add focused parser coverage.
   - Status: complete.
   - Cover import and simple control-flow ordering under the existing
     `analysis_ast_shadow` proof scope.

## Validation

```bash
cargo test -p tokmd-analysis --features ast ast --verbose
cargo clippy -p tokmd-analysis --features ast --all-targets -- -D warnings
cargo xtask doc-artifacts --check
cargo xtask docs --check
cargo xtask proof-policy --check
cargo xtask affected --base origin/main --head HEAD --json-output target/proof/affected-ast-rust-landmarks.json
cargo xtask proof --profile affected --base origin/main --head HEAD --plan --plan-json target/proof/proof-plan-ast-rust-landmarks.json --evidence-json target/proof/proof-evidence-ast-rust-landmarks.json
cargo xtask proof --profile affected --base origin/main --head HEAD --run-required --allow-local-required-execution --proof-run-summary target/proof/proof-run-summary-ast-rust-landmarks.json
cargo xtask proof-run-artifacts-check --proof-run-summary target/proof/proof-run-summary-ast-rust-landmarks.json
cargo fmt-check
cargo xtask publish-surface --json --verify-publish
git diff --check
```

## Stop Conditions

- Stop if AST parsing affects default product outputs.
- Stop if the implementation needs non-shadow receipt schema changes.
- Stop if the parser starts resolving imports semantically instead of recording
  landmarks.
- Stop if affected planning reports unknown files.
- Stop if docs imply proof promotion, Codecov upload, merge verdicts, or
  browser AST support.

## Checkpoint History

- 2026-05-14: Added Rust import landmarks and simple control-flow landmarks
  behind the existing `ast` feature. The new kinds feed only the shadow parser
  and artifact builder.
