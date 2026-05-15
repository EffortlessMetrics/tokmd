# Plan: AST Function-Boundary Corpus Expansion

- Status: active
- Related proposal:
- Related spec: `docs/specs/ast-shadow.md`
- Related ADR: `docs/adr/0008-ast-foundation.md`
- Related issues:

## Goal

Broaden AST shadow evidence for Rust function-boundary precision after the
first candidate-decision lane closed as `not yet`.

The previous lane proved that `cargo xtask ast-shadow-compare` and
`cargo xtask ast-shadow-check` can produce deterministic, verified evidence
from the repo-owned corpus manifest. It also classified the first corpus:
function-boundary mismatches were explainable, but the corpus was too small to
justify a public candidate proposal.

This lane should make the evidence less narrow while preserving shadow-mode
boundaries:

```text
broader explicit Rust corpus
  -> deterministic ast-shadow artifacts
  -> verifier receipt
  -> mismatch classification
  -> timing evidence
  -> public-candidate decision later
```

## Non-goals

- Do not add a public `tokmd ast`, `tokmd review`, or new product command.
- Do not change default `tokmd analyze`, `cockpit`, `context`, `handoff`,
  browser/WASM, FFI, Python, or Node outputs.
- Do not add public receipt fields or change schema meaning.
- Do not claim browser/WASM AST capability.
- Do not promote proof gates, scoped coverage, mutation, fast proof, or Codecov
  upload.
- Do not treat AST shadow diffs as merge verdicts, pass/fail proof, or review
  blockers.
- Do not implement cockpit or handoff AST integration from this lane.
- Do not expand into import or control-flow public-candidate decisions.

## Work Packets

1. Define corpus expansion categories.
   - Status: pending.
   - Record the file categories needed for a stronger decision: production
     code, tests, examples, macro-heavy files, generated-ish files,
     docs-adjacent Rust snippets, and parser-degraded fixtures.
2. Extend the repo-owned corpus manifest.
   - Status: pending.
   - Add explicit repo-relative Rust paths to `policy/ast-shadow-corpus.toml`.
   - Keep file reasons and expected signals specific.
   - Preserve absolute-path and path-escape rejection.
3. Collect verified expanded-corpus evidence.
   - Status: pending.
   - Run `ast-shadow-compare` and `ast-shadow-check` over the expanded manifest.
   - Record counts by landmark kind and function-boundary mismatch class.
4. Add candidate-corpus timing evidence.
   - Status: pending.
   - Use existing `tokmd.ast_shadow_perf.v1` evidence or add a scoped timing
     receipt for the expanded explicit corpus before making performance claims.
5. Reclassify function-boundary mismatches.
   - Status: pending.
   - Categorize heuristic-only and AST-only function landmarks using the
     buckets from `docs/specs/ast-shadow.md`.
6. Revisit the candidate decision.
   - Status: pending.
   - Choose one outcome: ready for public-candidate proposal, not yet, or
     shadow-only deferral.

## Validation

Docs-only slices should run:

```bash
cargo xtask doc-artifacts --check
cargo xtask docs --check
cargo xtask proof-policy --check
cargo xtask affected --base origin/main --head HEAD --json-output target/proof/affected-ast-function-boundary-corpus.json
cargo xtask proof --profile affected --base origin/main --head HEAD --plan --plan-json target/proof/proof-plan-ast-function-boundary-corpus.json --evidence-json target/proof/proof-evidence-ast-function-boundary-corpus.json
cargo fmt-check
git diff --check
```

Corpus, runner, or AST-code slices should also run the relevant focused proof:

```bash
cargo test -p tokmd-analysis --features ast ast --verbose
cargo run -p tokmd-analysis --features ast --example ast_shadow_perf -- --iterations 2 --files 2 --functions-per-file 3 --out target/perf/ast-shadow-perf.json
cargo test -p xtask ast_shadow --verbose
cargo xtask ast-shadow-compare --manifest policy/ast-shadow-corpus.toml --out target/tokmd-ast-shadow-corpus --summary-md target/tokmd-ast-shadow-corpus/summary.md
cargo xtask ast-shadow-check --manifest policy/ast-shadow-corpus.toml --dir target/tokmd-ast-shadow-corpus --json target/tokmd-ast-shadow-corpus/check.json
```

If public crate exports, dependencies, browser/WASM capability claims, schemas,
bindings, or package surfaces move, also run the relevant owner checks and
publish-surface verification.

## Stop Conditions

- Stop if the lane requires a public `tokmd` command before a public candidate
  proposal exists.
- Stop if AST evidence changes default product receipts or browser/WASM
  capability claims.
- Stop if parser degradation is hidden or counted as available proof.
- Stop if unsupported files are counted as successful AST evidence.
- Stop if control-flow or import evidence is promoted by piggybacking on the
  function-boundary decision.
- Stop if proof, scoped coverage, mutation, fast proof, or Codecov upload is
  promoted by this lane.
- Stop if evidencebus runtime implementation becomes necessary.
- Stop if affected planning reports unknown files.
- Stop if generated `target/` artifacts are staged or committed.

## Checkpoint History

- 2026-05-14: Started after the AST function-boundary candidate decision closed
  as `not yet`. The next evidence need is broader corpus coverage, not product
  integration.
