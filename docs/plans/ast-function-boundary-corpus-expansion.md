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
   - Status: complete.
   - Record the file categories needed for a stronger decision: production
     code, tests, examples, macro-heavy files, generated-ish files,
     docs-adjacent Rust snippets, and parser-degraded fixtures.
2. Extend the repo-owned corpus manifest.
   - Status: complete.
   - Add explicit repo-relative Rust paths to `policy/ast-shadow-corpus.toml`.
   - Keep file reasons and expected signals specific.
   - Preserve absolute-path and path-escape rejection.
3. Collect verified expanded-corpus evidence.
   - Status: complete.
   - Run `ast-shadow-compare` and `ast-shadow-check` over the expanded manifest.
   - Record counts by landmark kind and function-boundary mismatch class.
4. Add candidate-corpus timing evidence.
   - Status: complete.
   - Added an opt-in `cargo xtask ast-shadow-compare --timing-json <PATH>`
     receipt for the expanded explicit corpus.
   - Kept the receipt developer-facing and outside public `tokmd` CLI,
     default outputs, public receipts, browser/WASM capability, and proof
     promotion.
5. Reclassify function-boundary mismatches.
   - Status: complete.
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
cargo xtask ast-shadow-compare --manifest policy/ast-shadow-corpus.toml --out target/tokmd-ast-shadow-corpus --summary-md target/tokmd-ast-shadow-corpus/summary.md --timing-json target/tokmd-ast-shadow-corpus/timing.json
cargo xtask ast-shadow-check --dir target/tokmd-ast-shadow-corpus --json target/tokmd-ast-shadow-corpus/check.json
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
- 2026-05-14: Defined the corpus category taxonomy and extended
  `policy/ast-shadow-corpus.toml` with production, test, macro-heavy,
  generated-like, product-adjacent, shadow-implementation, and tooling entries.
- 2026-05-14: Collected verified expanded-corpus evidence from the 14-file
  manifest. Function landmarks had 225 matches, 24 heuristic-only entries,
  and 0 AST-only entries; one parser-degraded fixture remained explicit.
- 2026-05-15: Added scoped candidate-corpus timing evidence via
  `tokmd.ast_shadow_compare_timing.v1`. The timing receipt covers the explicit
  manifest corpus and remains developer-facing shadow evidence only.
- 2026-05-15: Reclassified expanded-corpus function-boundary mismatches after
  the timing slice. The corpus now reports 230 matched function landmarks, 25
  heuristic-only function landmarks, and 0 AST-only function landmarks. The
  heuristic-only rows remain explainable as embedded Rust source strings in
  tests/tooling plus the intentional parse-degraded fixture.

## Corpus Expansion Categories

The expanded manifest should make each selected Rust file explain why it is in
the function-boundary evidence set. A future public-candidate decision should
not rely on a corpus that is accidentally weighted toward parser internals or
clean fixtures.

| Category | Purpose | Expected signal |
| --- | --- | --- |
| `fixture_baseline` | Keep a small known-clean Rust file with ordinary imports, one function, and simple control flow. | Verifies runner, parser, and comparison plumbing without production noise. |
| `fixture_parse_degraded` | Keep at least one intentionally malformed Rust file. | Proves parse degradation remains explicit and is not counted as available AST proof. |
| `shadow_implementation` | Cover AST shadow parser, artifact builder, and comparison internals. | Keeps the evidence machinery itself represented without mistaking it for user-facing output. |
| `production_implementation` | Cover ordinary shipped Rust modules outside AST-specific code. | Shows function-boundary behavior on code users actually maintain. |
| `heuristic_implementation` | Cover the current heuristic extraction code and its tests. | Finds where heuristics over-report or under-report their own edge cases. |
| `test_corpus` | Cover unit or integration tests with helper functions and assertion-heavy bodies. | Separates real test helper functions from assertion text or embedded snippets. |
| `fixture_string_risk` | Cover files that embed Rust examples inside strings, raw strings, or docs-adjacent examples. | Measures the known heuristic false-positive class without treating it as production code. |
| `macro_heavy` | Cover files that use macro invocations, generated implementations, or declarative helper macros. | Identifies macro-shaped function-boundary disagreement before public claims. |
| `generated_like` | Cover checked-in generated-ish or mechanically repetitive Rust when present. | Checks whether repetition or generated style skews counts or ordering. |
| `product_adjacent_surface` | Cover review, handoff, cockpit, or context-selection code where function precision might later matter. | Keeps evidence connected to plausible user-visible surfaces without changing them. |
| `tooling_implementation` | Cover `xtask` runner/checker code that produces or verifies AST shadow evidence. | Proves the developer evidence tooling itself is represented in the corpus. |

Selection should stay explicit and repo-relative. Each manifest entry should
carry a narrow `reason`, a stable `category`, and an `expected_signal` that can
be checked after `ast-shadow-compare` and `ast-shadow-check` run. The expanded
corpus does not need every category in one PR, but the next candidate decision
should state which categories were present, which were absent, and why.

## Expanded Manifest Evidence

The first expanded manifest run used:

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

Verifier summary:

| Metric | Count |
| --- | ---: |
| Files | 14 |
| Matched landmarks | 443 |
| Heuristic-only landmarks | 208 |
| AST-only landmarks | 32 |
| Parse-degraded files | 1 |
| Unsupported files | 0 |

Landmark-kind summary:

| Kind | Matched | Heuristic-only | AST-only |
| --- | ---: | ---: | ---: |
| `control_flow` | 154 | 182 | 31 |
| `function` | 225 | 24 | 0 |
| `import` | 64 | 2 | 1 |

Function-boundary file summary:

| Path | Matched | Heuristic-only | AST-only | Parse degraded |
| --- | ---: | ---: | ---: | --- |
| `crates/tokmd-analysis/src/ast/rust.rs` | 13 | 5 | 0 | no |
| `crates/tokmd-analysis/src/ast/shadow.rs` | 27 | 6 | 0 | no |
| `crates/tokmd-analysis/src/complexity/functions/rust.rs` | 2 | 0 | 0 | no |
| `crates/tokmd-analysis/src/imports/parser.rs` | 35 | 2 | 0 | no |
| `crates/tokmd-cockpit/src/review_plan.rs` | 20 | 0 | 0 | no |
| `crates/tokmd-model/src/rows.rs` | 19 | 0 | 0 | no |
| `crates/tokmd/src/context_pack/select.rs` | 9 | 0 | 0 | no |
| `crates/tokmd/tests/data/large.rs` | 1 | 0 | 0 | no |
| `crates/tokmd/tests/handoff_integration.rs` | 17 | 2 | 0 | no |
| `fixtures/ast-shadow/rust/basic.rs` | 1 | 0 | 0 | no |
| `fixtures/ast-shadow/rust/parse-degraded.rs` | 1 | 1 | 0 | yes |
| `xtask/src/cli.rs` | 11 | 0 | 0 | no |
| `xtask/src/tasks/ast_shadow_check.rs` | 30 | 2 | 0 | no |
| `xtask/src/tasks/ast_shadow_compare.rs` | 39 | 6 | 0 | no |

Interpretation:

- Function-boundary AST-only misses remain at zero in the expanded corpus.
- Heuristic-only function entries remain real evidence to classify, not proof
  of readiness.
- The intentionally degraded fixture still appears as degraded evidence rather
  than available AST proof.
- Control-flow remains noisier than function boundaries and stays out of this
  candidate decision.

## Candidate-Corpus Timing Evidence

The scoped timing slice used the same expanded manifest and wrote a
developer-facing timing receipt:

```bash
cargo xtask ast-shadow-compare \
  --manifest policy/ast-shadow-corpus.toml \
  --out target/tokmd-ast-shadow-corpus \
  --summary-md target/tokmd-ast-shadow-corpus/summary.md \
  --timing-json target/tokmd-ast-shadow-corpus/timing.json

cargo xtask ast-shadow-check \
  --dir target/tokmd-ast-shadow-corpus \
  --json target/tokmd-ast-shadow-corpus/check.json
```

Receipt summary:

| Field | Value |
| --- | ---: |
| Schema | `tokmd.ast_shadow_compare_timing.v1` |
| Input files | 14 |
| Source bytes | 194799 |
| Matched landmarks | 453 |
| Heuristic-only landmarks | 210 |
| AST-only landmarks | 32 |
| Parse-degraded files | 1 |
| Unsupported files | 0 |
| Total comparison runtime | 116 ms |

Phase timings from the same local receipt:

| Phase | Duration |
| --- | ---: |
| Path selection | 2 ms |
| Input collection | 25 ms |
| Artifact build | 80 ms |
| Artifact write | 6 ms |
| Summary write | 0 ms |

The timing receipt records no timestamps, absolute paths, temporary
directories, or public product verdicts. It is suitable for candidate-corpus
evidence and proof routing, not a production performance budget. Duration
values are local observations and are expected to vary by machine, cache state,
and run. Counts differ from the first expanded manifest run because this slice
added timing-receipt code to `xtask/src/tasks/ast_shadow_compare.rs`, which is
itself part of the repo-owned corpus.

## Expanded Function-Boundary Reclassification

The reclassification used the same checked-in manifest and the same verified
artifact set:

```bash
cargo xtask ast-shadow-compare \
  --manifest policy/ast-shadow-corpus.toml \
  --out target/tokmd-ast-shadow-corpus \
  --summary-md target/tokmd-ast-shadow-corpus/summary.md \
  --timing-json target/tokmd-ast-shadow-corpus/timing.json

cargo xtask ast-shadow-check \
  --dir target/tokmd-ast-shadow-corpus \
  --json target/tokmd-ast-shadow-corpus/check.json
```

Function-boundary evidence:

| Measure | Count |
| --- | ---: |
| Matched function landmarks | 230 |
| Heuristic-only function landmarks | 25 |
| AST-only function landmarks | 0 |
| Parse-degraded files | 1 |
| Unsupported files | 0 |

Heuristic-only classification:

| Bucket | Count | Files |
| --- | ---: | --- |
| Embedded test, fixture, or generated source string | 24 | `crates/tokmd-analysis/src/ast/rust.rs`, `crates/tokmd-analysis/src/ast/shadow.rs`, `crates/tokmd-analysis/src/imports/parser.rs`, `crates/tokmd/tests/handoff_integration.rs`, `xtask/src/tasks/ast_shadow_check.rs`, `xtask/src/tasks/ast_shadow_compare.rs` |
| Malformed parse-degraded fixture | 1 | `fixtures/ast-shadow/rust/parse-degraded.rs` |
| Comment or documentation example false positive | 0 | None observed |
| Macro-ish pattern mismatch | 0 | None observed |
| Parser recovery mismatch in non-fixture code | 0 | None observed |
| Real heuristic false positive outside embedded source text | 0 | None observed |

AST-only classification:

| Bucket | Count | Files |
| --- | ---: | --- |
| Multi-line signature missed by heuristic | 0 | None observed |
| Visibility, async, unsafe, or extern shape missed by heuristic | 0 | None observed |
| Nested item missed by heuristic | 0 | None observed |
| Parser recovery case | 0 | None observed |
| Real heuristic miss | 0 | None observed |

Representative heuristic-only examples:

| Path | Lines | Name | Classification |
| --- | ---: | --- | --- |
| `crates/tokmd-analysis/src/ast/rust.rs` | 171, 174, 177, 205, 233-248 | `top_level`, `method`, `compute`, `ok` | Embedded parser test source strings |
| `crates/tokmd-analysis/src/ast/shadow.rs` | 372, 378, 416, 451, 469, 501 | `zed`, `top_level`, `ast_only`, `ok` | Embedded shadow-artifact test source strings |
| `crates/tokmd-analysis/src/imports/parser.rs` | 225, 234 | `main` | Import-parser fixture strings |
| `crates/tokmd/tests/handoff_integration.rs` | 256, 260 | `main`, `skip` | Temporary handoff fixture files written from string literals |
| `xtask/src/tasks/ast_shadow_check.rs` | 506, 542 | `fixture` | Verifier test fixture source strings |
| `xtask/src/tasks/ast_shadow_compare.rs` | 931-948, 973, 1005, 1036, 1079, 1083, 1225 | `compute`, `beta`, `alpha` | Runner test fixture source strings and manifest-order fixtures |
| `fixtures/ast-shadow/rust/parse-degraded.rs` | 3 | `broken` | Intentional malformed parse-degraded fixture |

Interpretation:

- The expanded corpus still shows zero AST-only function discoveries, so this
  slice does not prove AST improves function recall.
- The heuristic-only function set is concentrated in embedded source strings
  and the intentional degraded fixture, so AST remains useful as evidence
  against heuristic over-reporting.
- No real production-code heuristic false positives, macro-ish mismatches,
  comments/docs examples, or non-fixture parser recovery mismatches were
  observed in this corpus.
- The next decision should revisit whether the broader corpus, timing receipt,
  and mismatch classification justify a public-candidate proposal, another
  corpus expansion, or continued shadow-only deferral.
