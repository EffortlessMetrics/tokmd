# Plan: AST Shadow Performance Benchmark

- Status: complete
- Related proposal:
- Related spec: `docs/specs/ast-shadow.md`
- Related ADR: `docs/adr/0008-ast-foundation.md`
- Related issues:

## Goal

Add the first bounded performance benchmark for the feature-gated AST shadow
lane.

The benchmark should give maintainers a repeatable timing receipt for synthetic
Rust AST parsing and shadow artifact construction before any AST-derived facts
are proposed for public receipt fields or default workflows.

## Non-goals

- Do not add a public `tokmd ast` command.
- Do not run AST parsing in default `tokmd analyze`, `cockpit`, `context`, or
  `handoff` workflows.
- Do not change public receipt schemas or browser/WASM capability reporting.
- Do not promote proof gates, scoped coverage, mutation, or Codecov upload.
- Do not claim a production performance budget from synthetic-only evidence.
- Do not add an evidencebus runtime dependency.

## Work Packets

1. Add a developer-facing AST shadow performance example.
   - Status: complete.
   - `cargo run -p tokmd-analysis --features ast --example ast_shadow_perf`
     writes `tokmd.ast_shadow_perf.v1` JSON for a synthetic Rust corpus.
2. Record parser and artifact builder timings.
   - Status: complete.
   - The receipt records parse timing, shadow artifact construction timing,
     source/file counts, and observed landmark/artifact counts without raw
     repository paths.
3. Route the benchmark through AST proof scope.
   - Status: complete.
   - The `analysis_ast_shadow` proof scope runs a small bounded benchmark so
     AST shadow changes keep the timing path compiling and executable.

## Validation

```bash
cargo run -p tokmd-analysis --features ast --example ast_shadow_perf -- --iterations 2 --files 2 --functions-per-file 3 --out target/perf/ast-shadow-perf.json
cargo test -p tokmd-analysis --features ast ast --verbose
cargo clippy -p tokmd-analysis --features ast --all-targets -- -D warnings
cargo xtask doc-artifacts --check
cargo xtask docs --check
cargo xtask proof-policy --check
cargo xtask affected --base origin/main --head HEAD --json-output target/proof/affected-ast-shadow-perf.json
cargo xtask proof --profile affected --base origin/main --head HEAD --plan --plan-json target/proof/proof-plan-ast-shadow-perf.json --evidence-json target/proof/proof-evidence-ast-shadow-perf.json
cargo xtask proof --profile affected --base origin/main --head HEAD --run-required --allow-local-required-execution --proof-run-summary target/proof/proof-run-summary-ast-shadow-perf.json
cargo xtask proof-run-artifacts-check --proof-run-summary target/proof/proof-run-summary-ast-shadow-perf.json
cargo fmt-check
cargo xtask publish-surface --json --verify-publish
git diff --check
```

## Stop Conditions

- Stop if the benchmark changes default product output.
- Stop if the benchmark needs a production AST runner or public command.
- Stop if affected planning reports unknown files.
- Stop if docs imply proof promotion, Codecov upload, merge verdicts, or
  browser AST support.
- Stop if synthetic timing evidence is presented as a production performance
  envelope.

## Checkpoint History

- 2026-05-14: Added the first synthetic AST shadow performance receipt example
  and routed a small benchmark through the AST shadow proof scope.
