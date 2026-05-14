# Plan: AST Shadow Comparison Runner

- Status: active
- Related proposal:
- Related spec: `docs/specs/ast-shadow.md`
- Related ADR: `docs/adr/0008-ast-foundation.md`
- Related issues:

## Goal

Define and build the first developer-facing AST shadow comparison runner.

The runner should turn a small, explicit Rust source selection into
`tokmd.ast_shadow.v1` heuristic, AST, and diff artifacts so maintainers can
start collecting heuristic-vs-AST comparison evidence on real code before any
public receipt, default workflow, browser, or review-packet behavior changes.

The first comparison target is landmark presence by normalized path, landmark
kind, and stable identifier for Rust functions, imports, and simple
control-flow landmarks. This target is intentionally lower risk than semantic
equivalence, call graphs, type resolution, or complexity replacement.

## Non-goals

- Do not add a public `tokmd ast` or `tokmd review` command.
- Do not change default `tokmd analyze`, `cockpit`, `context`, `handoff`, FFI,
  Python, Node, or WASM outputs.
- Do not add public receipt fields or change existing schema meaning.
- Do not claim browser/WASM AST capability.
- Do not promote proof gates, scoped coverage, mutation, or Codecov upload.
- Do not build mergecode-style semantic graphs, call graphs, or dependency
  relationships.
- Do not treat shadow diffs as merge verdicts.

## Work Packets

1. Close the source-of-truth gap for the next AST lane.
   - Status: active.
   - Archive the completed AST shadow performance benchmark goal.
   - Retarget `.jules/goals/active.toml` to this comparison-runner lane.
   - Keep `docs/NEXT.md` and `docs/specs/ast-shadow.md` aligned with the
     runner boundary.
2. Add the first runner behind developer tooling.
   - Status: active.
   - Prefer `cargo xtask ast-shadow-compare` for the first runner so the public
     `tokmd` CLI stays unchanged while the artifact contract stabilizes.
   - Inputs should be explicit repo-relative Rust source paths and an output
     directory, with no network, GitHub, Codecov, or evidencebus dependency.
3. Generate the existing artifact set.
   - Status: active.
   - Reuse the `tokmd-analysis` AST shadow artifact builder to write
     `heuristic.json`, `ast.json`, and `diff.json`.
   - Avoid timestamps, absolute paths, temporary directories, and
     nondeterministic ordering.
4. Add a small fixture corpus and focused proof.
   - Status: active.
   - Cover a Rust fixture with functions, imports, and simple control flow.
   - Route the runner through the existing `analysis_ast_shadow` proof scope.
5. Collect comparison evidence without product adoption.
   - Status: pending.
   - Use runner output to decide whether function, import, or control-flow
     landmarks are accurate enough for a later public schema proposal.

## Validation

For the docs-only source-of-truth slice:

```bash
cargo xtask doc-artifacts --check
cargo xtask docs --check
cargo xtask proof-policy --check
cargo xtask affected --base origin/main --head HEAD --json-output target/proof/affected-ast-shadow-runner-plan.json
cargo xtask proof --profile affected --base origin/main --head HEAD --plan --plan-json target/proof/proof-plan-ast-shadow-runner-plan.json --evidence-json target/proof/proof-evidence-ast-shadow-runner-plan.json
cargo fmt-check
git diff --check
```

For the later runner implementation slice, add:

```bash
cargo test -p tokmd-analysis --features ast ast --verbose
cargo run -p tokmd-analysis --features ast --example ast_shadow_perf -- --iterations 2 --files 2 --functions-per-file 3 --out target/perf/ast-shadow-perf.json
cargo test -p xtask ast_shadow --verbose
cargo xtask ast-shadow-compare --out target/tokmd-ast-shadow --path <fixture-rust-path>
cargo xtask publish-surface --json --verify-publish
```

## Stop Conditions

- Stop if the runner requires a new public `tokmd` command.
- Stop if AST shadow output changes default product receipts or browser/WASM
  capabilities.
- Stop if the runner needs network, GitHub Actions, Codecov, or evidencebus
  runtime dependencies.
- Stop if comparison artifacts include timestamps, absolute paths, or
  environment-specific temporary directories.
- Stop if affected planning reports unknown files.
- Stop if docs imply proof promotion, Codecov upload, merge verdicts, or
  default AST adoption.

## Checkpoint History

- 2026-05-14: Started the comparison-runner lane after the synthetic AST shadow
  performance receipt landed. The lane selects landmark presence for Rust
  functions, imports, and simple control-flow as the first comparison target and
  keeps the first runner in developer tooling rather than the public `tokmd`
  CLI.
- 2026-05-14: Added the first `cargo xtask ast-shadow-compare` runner slice.
  It accepts explicit repo-relative Rust paths, writes the existing
  `heuristic.json`, `ast.json`, and `diff.json` artifacts, and routes the
  runner plus fixture corpus through `analysis_ast_shadow` proof. The slice does
  not add a public `tokmd` CLI command or change default receipt behavior.
