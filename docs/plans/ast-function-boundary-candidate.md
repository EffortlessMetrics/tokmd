# Plan: AST Function-Boundary Candidate Evidence

- Status: active
- Related proposal:
- Related spec: `docs/specs/ast-shadow.md`
- Related ADR: `docs/adr/0008-ast-foundation.md`
- Related issues:

## Goal

Decide, with repeatable shadow evidence, whether AST-backed Rust
function-boundary facts are ready to move from developer-facing comparison
artifacts into a future public candidate surface.

The current AST shadow runner can already generate and verify
`tokmd.ast_shadow.v1` heuristic, AST, and diff artifacts for explicit Rust
files. This lane uses those artifacts to define the evidence bar for one fact
family before any default product behavior, public receipt schema, cockpit
output, handoff output, browser capability, or proof gate changes.

The target decision is intentionally narrow:

```text
Are Rust function-boundary facts accurate, explainable, performant, and
fallback-safe enough to justify a later public candidate proposal?
```

The answer may be yes, no, or not yet. It must be backed by checked shadow
artifacts, corpus notes, mismatch classification, and timing evidence.

## Non-goals

- Do not add a public `tokmd ast`, `tokmd review`, or new product command.
- Do not change default `tokmd analyze`, `cockpit`, `context`, `handoff`,
  browser/WASM, FFI, Python, or Node outputs.
- Do not add public receipt fields or change schema meaning.
- Do not claim browser/WASM AST capability.
- Do not promote proof gates, scoped coverage, mutation, fast proof, or Codecov
  upload.
- Do not build evidencebus runtime export or make tokmd carry evidencebus
  responsibilities.
- Do not build mergecode-style semantic graphs, call graphs, type resolution,
  or cross-file semantic relationships.
- Do not treat AST shadow diffs as merge verdicts, pass/fail proof, or review
  blockers.
- Do not implement cockpit or handoff AST integration before the candidate
  evidence and contract justify it.

## Work Packets

1. Define the function-boundary candidate evidence bar.
   - Status: complete.
   - Record what evidence must exist before a public candidate proposal can be
     drafted.
   - Keep this first slice docs/control-plane only.
2. Make the comparison corpus repeatable.
   - Status: complete.
   - Added `policy/ast-shadow-corpus.toml`, a repo-owned draft corpus manifest
     with explicit repo-relative Rust paths, selection reasons, and expected
     evidence signals.
   - The first corpus includes fixtures, AST implementation code, heuristic
     implementation code, parser code with fixture-string risk, review-surface
     logic, agent-context selection logic, and the comparison runner.
3. Let the runner consume the corpus manifest.
   - Status: complete.
   - `cargo xtask ast-shadow-compare --manifest policy/ast-shadow-corpus.toml`
     now expands the repo-owned corpus manifest into the same deterministic
     `heuristic.json`, `ast.json`, `diff.json`, and optional `summary.md`
     artifacts as explicit `--path` mode.
   - Preserve existing explicit `--path` mode.
   - Keep manifest paths repo-relative, Rust-only, sorted, and rejected when
     absolute or escaping the repository.
4. Collect and classify function-boundary mismatch evidence.
   - Status: pending.
   - Run `ast-shadow-compare` and `ast-shadow-check` over the manifest corpus.
   - Categorize heuristic-only function discoveries separately from AST-only
     discoveries.
   - Distinguish fixture-string false positives from comments/docs examples,
     macro-ish patterns, malformed input, parser recovery, and true heuristic
     misses or false positives.
5. Define promotion criteria as a spec-level decision framework.
   - Status: pending.
   - Use the checked corpus evidence to define what would justify public
     candidate work.
   - Keep the framework advisory until maintainers explicitly accept a product
     proposal.
6. Draft a public candidate proposal only if evidence supports it.
   - Status: pending.
   - The proposal must identify the affected schema family, fallback behavior,
     browser/WASM reporting, proof ownership, rollback plan, and first product
     surface.
   - A likely first product surface is optional cockpit or handoff evidence,
     not default `analyze`.
7. Close the lane with a durable decision.
   - Status: pending.
   - Record whether function boundaries are ready for a public candidate
     proposal, need more corpus evidence, or should remain developer-only
     shadow evidence.

## Candidate Evidence Criteria

Before function-boundary facts can move toward a public candidate surface, the
lane needs evidence for all of the following:

- The corpus is repeatable from a checked-in manifest and covers more than the
  AST implementation files.
- `cargo xtask ast-shadow-compare` produces deterministic artifact bytes for
  the corpus.
- `cargo xtask ast-shadow-check` accepts the generated artifacts and verifies
  schema, relative paths, sorted entries, timestamp-free content, and summary
  counts.
- Parse degradation is zero or each degraded file is explained and categorized.
- Unsupported files are explicit and not counted as successful AST evidence.
- Function-kind by-kind counts are recorded separately from import and
  control-flow counts.
- Heuristic-only function landmarks are inspected and categorized as fixture
  strings, comments/docs examples, macro-ish patterns, malformed input, parser
  mismatch, or real heuristic false positives.
- AST-only function landmarks are inspected and categorized as multi-line
  signatures, visibility/async/unsafe/extern shapes, nested items, parser
  recovery cases, or real heuristic misses.
- Timing is bounded with `tokmd.ast_shadow_perf.v1` evidence or a clearly
  scoped equivalent runner receipt.
- Fallback behavior is documented for unsupported languages, unavailable AST
  builds, and browser/WASM.
- Any later public schema impact is additive or explicitly versioned, and the
  affected schema family is named before implementation starts.

## Validation

Docs-only slices should run:

```bash
cargo xtask doc-artifacts --check
cargo xtask docs --check
cargo xtask proof-policy --check
cargo xtask affected --base origin/main --head HEAD --json-output target/proof/affected-ast-function-boundary-candidate.json
cargo xtask proof --profile affected --base origin/main --head HEAD --plan --plan-json target/proof/proof-plan-ast-function-boundary-candidate.json --evidence-json target/proof/proof-evidence-ast-function-boundary-candidate.json
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

- Stop if the lane requires a public `tokmd` command before the evidence
  decision exists.
- Stop if AST evidence changes default product receipts or browser/WASM
  capability claims.
- Stop if a proposed public field lacks an identified schema family and
  fallback story.
- Stop if AST shadow artifacts include timestamps, absolute paths, temporary
  directories, or nondeterministic ordering.
- Stop if parser degradation is hidden or counted as available proof.
- Stop if control-flow or import evidence is promoted by piggybacking on the
  function-boundary decision.
- Stop if proof, scoped coverage, mutation, fast proof, or Codecov upload is
  promoted by this lane.
- Stop if evidencebus runtime implementation becomes necessary.
- Stop if affected planning reports unknown files.
- Stop if generated `target/` artifacts are staged or committed.

## Checkpoint History

- 2026-05-14: Started after the AST shadow comparison-runner lane closed
  through first enforcement. Existing evidence shows function-boundary
  mismatches are the narrowest first candidate; control-flow remains noisier
  and shadow-only.
- 2026-05-14: Added the draft corpus manifest in
  `policy/ast-shadow-corpus.toml` and routed it through the
  `analysis_ast_shadow` proof scope. The manifest is repo-owned input for a
  later runner-consumption slice; it does not change public tokmd behavior.
- 2026-05-14: Extended `cargo xtask ast-shadow-compare` to consume the corpus
  manifest while preserving explicit `--path` mode. The manifest runner stays
  developer-facing and keeps AST shadow output out of public tokmd workflows.
