# Plan: AST Shadow Artifact Builder

- Status: complete
- Related proposal:
- Related spec: `docs/specs/ast-shadow.md`
- Related ADR: `docs/adr/0008-ast-foundation.md`
- Related issues:

## Goal

Add the first feature-gated AST shadow artifact builder inside
`tokmd-analysis` so later runners can write deterministic
`target/tokmd-ast-shadow/{heuristic,ast,diff}.json` comparison evidence.

The builder should make artifact emission real without changing default
receipts, browser capabilities, bindings, CI gate behavior, proof promotion, or
public product commands.

## Non-goals

- Do not add a `tokmd ast` command or any other product CLI surface.
- Do not run AST parsing in default `tokmd analyze`, `cockpit`, `context`, or
  `handoff` workflows.
- Do not change public receipt schemas or browser/WASM capability reporting.
- Do not promote proof gates, scoped coverage, mutation, or Codecov upload.
- Do not choose the final heuristic fact family for production comparison.
- Do not add an evidencebus runtime dependency.

## Work Packets

1. Add AST shadow artifact DTO/building helpers.
   - Status: complete.
   - Accept normalized repository-relative paths, Rust source text, and
     caller-supplied heuristic landmarks.
   - Produce deterministic JSON values for `heuristic.json`, `ast.json`, and
     `diff.json`.
2. Add artifact writing helpers.
   - Status: complete.
   - Write the three contract artifact names under a caller-provided output
     directory, using stable pretty JSON and no timestamps.
3. Preserve shadow-only behavior.
   - Status: complete.
   - Keep the builder behind the existing `ast` feature and out of default
     workflows.
4. Add focused tests.
   - Status: complete.
   - Cover deterministic ordering, path normalization, parse-degraded files,
     and artifact filenames.

## Validation

```bash
cargo test -p tokmd-analysis --features ast ast --verbose
cargo clippy -p tokmd-analysis --features ast --all-targets -- -D warnings
cargo xtask doc-artifacts --check
cargo xtask docs --check
cargo xtask proof-policy --check
cargo xtask affected --base origin/main --head HEAD --json-output target/proof/affected-ast-shadow.json
cargo xtask proof --profile affected --base origin/main --head HEAD --plan --plan-json target/proof/proof-plan-ast-shadow.json --evidence-json target/proof/proof-evidence-ast-shadow.json
cargo xtask proof --profile affected --base origin/main --head HEAD --run-required --allow-local-required-execution --proof-run-summary target/proof/proof-run-summary-ast-shadow.json
cargo xtask proof-run-artifacts-check --proof-run-summary target/proof/proof-run-summary-ast-shadow.json
cargo fmt-check
git diff --check
```

## Stop Conditions

- Stop if the implementation changes default receipt output or CLI behavior.
- Stop if browser/WASM AST capability is implied without capability evidence.
- Stop if affected planning reports unknown files.
- Stop if AST shadow artifacts include timestamps, absolute paths, or
  nondeterministic ordering.
- Stop if the builder starts making merge, gate-promotion, or evidencebus
  claims.

## Checkpoint History

- 2026-05-14: Added the first `tokmd-analysis` AST shadow artifact builder and
  writer. It can build and write the three developer-facing shadow artifacts
  from supplied heuristic landmarks plus Rust Tree-sitter landmarks, while
  leaving default tokmd workflows unchanged.
