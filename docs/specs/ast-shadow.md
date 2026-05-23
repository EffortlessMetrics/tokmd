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
- a repo-owned corpus manifest such as `policy/ast-shadow-corpus.toml` for
  repeatable evidence collection;
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
  summary.md         # optional human summary
  timing.json        # optional scoped comparison timing receipt
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

The comparison runner may also write an optional `summary.md`:

```bash
cargo xtask ast-shadow-compare \
  --path fixtures/ast-shadow/rust/basic.rs \
  --out target/tokmd-ast-shadow \
  --summary-md target/tokmd-ast-shadow/summary.md
```

For repeatable candidate evidence, the runner may consume the repo-owned
corpus manifest instead of listing every file on the command line:

```bash
cargo xtask ast-shadow-compare \
  --manifest policy/ast-shadow-corpus.toml \
  --out target/tokmd-ast-shadow \
  --summary-md target/tokmd-ast-shadow/summary.md \
  --timing-json target/tokmd-ast-shadow/timing.json
```

The Markdown summary is a human review layer over `diff.json`. It should include
aggregate counts, mismatch counts by landmark kind, per-file comparison status,
artifact paths, and a reproduction command. It must not add pass/fail language,
merge verdicts, proof-promotion claims, or public receipt semantics.

The optional comparison timing receipt is developer-facing xtask evidence:

```bash
cargo xtask ast-shadow-compare \
  --manifest policy/ast-shadow-corpus.toml \
  --out target/tokmd-ast-shadow \
  --summary-md target/tokmd-ast-shadow/summary.md \
  --timing-json target/tokmd-ast-shadow/timing.json
```

It emits `tokmd.ast_shadow_compare_timing.v1` for the explicit corpus selected
by `--path` or `--manifest`. The receipt records repo-relative artifact paths,
input-file and source-byte counts, diff summary counts, and bounded phase
durations for path selection, input collection, artifact construction, artifact
writing, summary writing, and total comparison runtime. It must not include
timestamps, absolute paths, temporary directories, environment variables,
pass/fail product language, merge verdicts, proof-promotion claims, or public
receipt semantics.

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
accept the same explicit repo-relative Rust `--path` inputs or repo-relative
`--manifest` input as the comparison runner. In that mode it regenerates the
three shadow artifacts into `--dir` before validating them.

The first implementation lives in `tokmd-analysis` behind the existing `ast`
feature. It builds and writes the three artifact JSON files for caller-provided
Rust source and heuristic landmarks. Its Rust parser records function, import,
and simple control-flow landmarks, but it is not wired into default CLI,
receipt, browser, FFI, Python, Node, or CI behavior.

The comparison-runner implementation plan,
`docs/plans/ast-shadow-comparison-runner.md`, is complete through first
enforcement. The follow-on function-boundary candidate and corpus-expansion
plans, `docs/plans/ast-function-boundary-candidate.md` and
`docs/plans/ast-function-boundary-corpus-expansion.md`, are also complete. Both
closed with a `not yet` outcome for public function-boundary adoption, leaving
AST evidence in shadow mode.

The latest AST boundary is: there is no active AST productization lane. Future
AST work should start from a fresh proposal that names the product surface,
schema family, fallback behavior, proof ownership, browser/WASM reporting, and
rollback story before implementation. The existing runner, verifier, Markdown
summary, corpus manifest, mismatch classification, and timing receipts remain
developer-facing shadow evidence only. They do not change default `tokmd` CLI
behavior, public receipts, browser/WASM capability, cockpit output, handoff
output, proof gates, Codecov defaults, or evidencebus runtime behavior.

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

## Function-Boundary Candidate Promotion Criteria

Function-boundary precision is the first AST-backed fact family eligible for a
future public-candidate proposal. Eligibility does not mean automatic adoption.
It means maintainers have enough shadow evidence to decide whether to draft a
product proposal for one explicit surface.

A function-boundary public-candidate proposal may be drafted only when all of
the following are true:

- the comparison corpus is repeatable from checked-in repo-relative inputs,
  currently `policy/ast-shadow-corpus.toml`;
- `cargo xtask ast-shadow-check` accepts regenerated artifacts for that corpus;
- every parse-degraded file is explicit, expected, or categorized separately
  from available AST evidence;
- unsupported files are explicit and never counted as passing AST evidence;
- function-kind counts are separated from import and control-flow counts;
- heuristic-only function landmarks are inspected and categorized as embedded
  fixture or test-source strings, comment/doc examples, macro-ish patterns,
  malformed input, parser mismatch, or real heuristic false positives;
- AST-only function landmarks are inspected and categorized as multi-line
  signatures, visibility/async/unsafe/extern shapes, nested items, parser
  recovery cases, or real heuristic misses;
- the evidence shows a user-visible problem that AST can improve, such as
  reducing review or handoff noise from heuristic function over-reporting;
- no unexplained AST regression or parser degradation remains in the candidate
  corpus;
- timing evidence is recorded with `tokmd.ast_shadow_perf.v1` or a scoped
  equivalent before making performance claims;
- fallback behavior is explicit for builds without the `ast` feature,
  unsupported languages, parser degradation, and browser/WASM;
- the affected public schema family is named before implementation starts;
- the proposed schema impact is additive or explicitly versioned;
- proof ownership is mapped in `ci/proof.toml`; and
- rollback is possible by disabling the candidate surface without changing
  existing heuristic receipts.

The candidate decision must choose one of these outcomes:

- **ready for proposal**: evidence supports drafting a public-candidate
  proposal for a named surface and schema family;
- **not yet**: evidence is useful but needs a larger corpus, better
  classification, timing evidence, or fallback design;
- **shadow-only**: AST evidence remains developer/review evidence and should
  not move toward a public surface.

The first manifest-corpus classification clears repeatability,
artifact-verifier, and mismatch-classification evidence for the initial corpus.
It does not by itself clear public-candidate promotion, because a future
proposal still needs the schema family, fallback policy, timing envelope,
rollback path, and first product surface named explicitly.

## Proof Requirements

Any PR that changes the AST shadow contract, AST parser code, or shadow artifact
names should run:

```bash
cargo test -p tokmd-analysis --features ast ast --verbose
cargo run -p tokmd-analysis --features ast --example ast_shadow_perf -- --iterations 2 --files 2 --functions-per-file 3 --out target/perf/ast-shadow-perf.json
cargo test -p xtask ast_shadow --verbose
cargo xtask ast-shadow-compare --manifest policy/ast-shadow-corpus.toml --out target/tokmd-ast-shadow --summary-md target/tokmd-ast-shadow/summary.md --timing-json target/tokmd-ast-shadow/timing.json
cargo xtask ast-shadow-check --dir target/tokmd-ast-shadow --json target/tokmd-ast-shadow/check.json
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

The first public-candidate fact family is function-boundary precision.

That does not mean AST-backed function facts are accepted into public receipts.
It means future evidence collection should evaluate function-boundary precision
first because the initial internal-corpus comparison showed explainable
heuristic over-reporting in fixture strings and examples, no parser degradation,
and no AST-only landmark discovery on the selected files. Function boundaries
are also easier to review, reproduce, and map to later cockpit or handoff
signals than richer import semantics or control-flow landmarks.

Open questions before any public schema or default behavior proposal:

- What corpus size and repository mix are enough to judge function-boundary
  precision across production code, tests, examples, macros, generated files,
  and degraded parses.
- What timing envelope is acceptable for function-boundary AST evidence when
  compared with current heuristic output.
- Whether the eventual public change is an additive shadow-derived receipt
  field, a new artifact reference, or no product change.
- How browser/WASM, bindings, and unsupported languages should report fallback
  behavior if function-boundary evidence later graduates from shadow mode.
