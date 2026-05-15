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
   - Status: complete.
   - Ran `ast-shadow-compare` and `ast-shadow-check` over the manifest corpus.
   - Categorized heuristic-only function discoveries separately from AST-only
     discoveries.
   - Distinguished fixture-string false positives from comments/docs examples,
     macro-ish patterns, malformed input, parser recovery, and true heuristic
     misses or false positives.
5. Define promotion criteria as a spec-level decision framework.
   - Status: complete.
   - Used the checked corpus evidence to define what would justify public
     candidate work.
   - Kept the framework advisory until maintainers explicitly accept a product
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

## Manifest Corpus Evidence

The first repeatable manifest-corpus comparison was collected with:

```bash
cargo xtask ast-shadow-compare \
  --manifest policy/ast-shadow-corpus.toml \
  --out target/tokmd-ast-shadow-corpus \
  --summary-md target/tokmd-ast-shadow-corpus/summary.md
cargo xtask ast-shadow-check \
  --manifest policy/ast-shadow-corpus.toml \
  --dir target/tokmd-ast-shadow-corpus \
  --json target/tokmd-ast-shadow-corpus/check.json
```

The verifier accepted the generated artifacts with:

| Measure | Count |
| --- | ---: |
| Files | 9 |
| Matched landmarks | 286 |
| Heuristic-only landmarks | 106 |
| AST-only landmarks | 31 |
| Parse-degraded files | 1 |
| Unsupported files | 0 |

Function landmarks were the narrow candidate signal inside the broader
landmark comparison:

| Function-boundary measure | Count |
| --- | ---: |
| Matched function landmarks | 147 |
| Heuristic-only function landmarks | 20 |
| AST-only function landmarks | 0 |

The observed heuristic-only function landmarks were:

| Bucket | Count | Files |
| --- | ---: | --- |
| Fixture or test-source string false positive | 19 | `crates/tokmd-analysis/src/ast/rust.rs`, `crates/tokmd-analysis/src/ast/shadow.rs`, `crates/tokmd-analysis/src/imports/parser.rs`, `xtask/src/tasks/ast_shadow_compare.rs` |
| Malformed parse-degraded fixture | 1 | `fixtures/ast-shadow/rust/parse-degraded.rs` |
| Comment or documentation example false positive | 0 | None observed |
| Macro-ish pattern mismatch | 0 | None observed |
| Parser recovery mismatch in non-fixture code | 0 | None observed |
| Real heuristic false positive outside embedded source text | 0 | None observed |

The observed AST-only function landmarks were:

| Bucket | Count | Files |
| --- | ---: | --- |
| Multi-line signature missed by heuristic | 0 | None observed |
| Visibility, async, unsafe, or extern shape missed by heuristic | 0 | None observed |
| Nested item missed by heuristic | 0 | None observed |
| Parser recovery case | 0 | None observed |
| Real heuristic miss | 0 | None observed |

This is useful shadow evidence, not a product decision. The manifest corpus
shows where the AST view avoids heuristic over-reporting from embedded Rust
source strings in tests and keeps malformed input visible through parse
degradation. It does not yet prove that public receipt fields should change,
because the corpus produced no AST-only function discoveries and still needs
promotion criteria, timing evidence review, fallback policy, and schema impact
analysis before any public candidate proposal.

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
- 2026-05-14: Classified the first manifest-corpus function-boundary mismatch
  evidence. The checked corpus produced 147 matched function landmarks, 20
  heuristic-only function landmarks, and 0 AST-only function landmarks. The
  heuristic-only set was explained by embedded fixture/test source strings plus
  the intentional parse-degraded fixture, so function boundaries remain a
  promising public-candidate fact family but are not ready for product
  integration without promotion criteria and fallback/schema analysis.
- 2026-05-14: Added the spec-level function-boundary promotion criteria to
  `docs/specs/ast-shadow.md`. The framework requires repeatable corpus
  evidence, verifier acceptance, mismatch categorization, timing evidence,
  fallback policy, schema-family identification, proof ownership, and rollback
  before any public candidate proposal can claim readiness.
