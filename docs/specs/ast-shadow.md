# Spec: AST Shadow Artifacts

- Status: draft
- Schema family, if any: `tokmd.ast_shadow.v1`
- Related ADRs: `docs/adr/0008-ast-foundation.md`
- Related proof scopes: `analysis_ast_shadow`

## Contract

AST shadow artifacts are developer-facing comparison evidence for future
language-aware analysis. They exist to compare current heuristic facts with
feature-gated AST facts without changing default `tokmd` receipts, schemas,
browser capabilities, bindings, or CI gates.

During shadow mode:

- default `tokmd analyze`, `tokmd cockpit`, `tokmd context`, `tokmd handoff`,
  FFI, Python, Node, and WASM outputs must remain unchanged;
- AST parsing must stay behind the explicit `ast` feature;
- Rust is the only parser-backed language until comparison evidence justifies a
  later language slice;
- generated shadow artifacts are not merge verdicts, proof promotion receipts,
  or evidencebus packets;
- any future public receipt field that changes meaning because of AST evidence
  requires schema-family review before adoption.

## Inputs

The first shadow slice may read:

- normalized repository-relative source paths;
- Rust source text for files selected by a future shadow runner;
- heuristic facts already produced by existing analysis modules;
- AST capability metadata from `tokmd-analysis` when built with
  `--features ast`.

The shadow path must not require:

- network access;
- GitHub Actions metadata;
- Codecov upload;
- evidencebus runtime dependencies;
- browser/WASM AST support.

## Outputs

The stable developer-facing output directory is:

```text
target/tokmd-ast-shadow/
  heuristic.json
  ast.json
  diff.json
  check.json          # optional verifier receipt
```

The artifact set uses schema family `tokmd.ast_shadow.v1`.

`heuristic.json` should record the existing heuristic facts selected for
comparison, including normalized paths and stable identifiers. The first
library builder accepts caller-supplied heuristic landmarks; choosing the
production heuristic source remains a later runner decision.

`ast.json` should record parser-backed Rust facts selected for comparison,
including parser capability metadata, normalized paths, function/import/simple
control-flow landmarks, parser status, and recoverable parse-error state.

`diff.json` should record deterministic comparison results between heuristic
and AST facts. It should distinguish exact matches, AST-only facts,
heuristic-only facts, parse-degraded files, and unsupported files.
It also includes a top-level `summary` object with aggregate counts for files,
matched landmarks, heuristic-only landmarks, AST-only landmarks,
parse-degraded files, and unsupported files so maintainers can judge a
comparison without scanning every file entry.

All three artifacts must avoid timestamps, absolute paths, environment-specific
temporary directories, and nondeterministic ordering.

The verifier is developer-facing xtask tooling:

```bash
cargo xtask ast-shadow-check \
  --dir target/tokmd-ast-shadow \
  --json target/tokmd-ast-shadow/check.json
```

It emits `tokmd.ast_shadow_check.v1` when requested. The checker verifies that
the three shadow artifacts exist, use the expected schema and kind, keep
repo-relative sorted paths, avoid timestamp and environment-specific strings,
and report summary counts that match the per-file diff entries. The check
receipt is verifier evidence only; it is not a public `tokmd` receipt, merge
verdict, proof promotion signal, browser capability claim, or evidencebus
packet.

For proof commands that need to be self-contained, `ast-shadow-check` may also
accept the same explicit repo-relative Rust `--path` inputs as the comparison
runner. In that mode it regenerates the three shadow artifacts into `--dir`
before validating them.

The first implementation lives in `tokmd-analysis` behind the existing `ast`
feature. It builds and writes the three artifact JSON files for caller-provided
Rust source and heuristic landmarks. Its Rust parser records function, import,
and simple control-flow landmarks, but it is not wired into default CLI,
receipt, browser, FFI, Python, Node, or CI behavior.

The next active implementation plan is
`docs/plans/ast-shadow-comparison-runner.md`. It selects Rust landmark presence
for functions, imports, and simple control-flow as the first comparison target
and keeps the first runner in developer tooling rather than the public `tokmd`
CLI while the artifact contract stabilizes.

`tokmd-analysis` also provides a developer-facing synthetic performance
example:

```bash
cargo run -p tokmd-analysis --features ast --example ast_shadow_perf -- \
  --out target/perf/ast-shadow-perf.json
```

It emits `tokmd.ast_shadow_perf.v1` timing evidence for Rust landmark parsing
and shadow artifact construction. This is benchmark evidence only; it is not a
public receipt schema, merge verdict, production performance budget, or default
workflow.

## Compatibility

AST shadow artifacts are intentionally outside the public receipt contract.
Existing receipt schemas remain authoritative:

- core receipts stay under `tokmd-types`;
- analysis receipts stay under `tokmd-analysis-types`;
- cockpit receipts stay under `tokmd-types`;
- context and handoff schemas stay under `tokmd-types`.

Shadow artifacts may be versioned independently. A future migration from
shadow evidence into public receipts must:

- identify the affected schema family;
- explain whether the new field is additive or changes existing meaning;
- preserve heuristic fallback for unsupported languages and runtimes;
- keep browser/WASM capability reporting honest;
- update proof scopes before public behavior changes.

## Proof Requirements

Any PR that changes the AST shadow contract, AST parser code, or shadow artifact
names should run:

```bash
cargo test -p tokmd-analysis --features ast ast --verbose
cargo run -p tokmd-analysis --features ast --example ast_shadow_perf -- --iterations 2 --files 2 --functions-per-file 3 --out target/perf/ast-shadow-perf.json
cargo test -p xtask ast_shadow --verbose
cargo xtask ast-shadow-compare --path fixtures/ast-shadow/rust/basic.rs --out target/tokmd-ast-shadow
cargo xtask ast-shadow-check --path fixtures/ast-shadow/rust/basic.rs --dir target/tokmd-ast-shadow --json target/tokmd-ast-shadow/check.json
cargo xtask proof-policy --check
cargo xtask affected --base origin/main --head HEAD --json-output target/proof/affected-ast-shadow.json
cargo xtask proof --profile affected --base origin/main --head HEAD --plan --plan-json target/proof/proof-plan-ast-shadow.json --evidence-json target/proof/proof-evidence-ast-shadow.json
cargo fmt-check
git diff --check
```

If the change touches public crate exports, dependencies, schemas, browser/WASM
capabilities, or package surfaces, also run the relevant owner checks, including
publish-surface verification when package/public API boundaries move.

## Open Questions

- Which existing heuristic fact family should be the first full
  heuristic-vs-AST comparison target after the initial landmark-presence runner:
  complexity landmarks, richer imports, or function-boundary precision?
- What corpus size and performance envelope are required before any AST-derived
  public receipt field is proposed.
