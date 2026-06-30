# Spec: AST Shadow Backend Identity

- Status: draft
- Schema family, if any: n/a (identity vocabulary spanning `tokmd.ast_shadow.v1` and `tokmd.syntax_receipt.v1`)
- Related ADRs: `docs/adr/0008-ast-foundation.md`
- Related proof scopes: `analysis_ast_shadow`

## Contract

AST shadow evidence compares facts that come from different *fact backends*.
Today two backends exist and one identifier is reserved for a future backend:

| `backend_id` | State | Source of truth |
| --- | --- | --- |
| `heuristic` | implemented, always available | existing deterministic line/text heuristics in `tokmd-analysis` |
| `tree-sitter` | implemented, feature-gated behind `ast` | locked Tree-sitter grammars in `crates/tokmd-analysis/src/ast/registry.rs` and `crates/tokmd-analysis/src/ast/capability.rs` |
| `adze-proposed` | reserved name only; not implemented; not on the current roadmap | see the claim boundary below |

This spec defines a durable *backend identity vocabulary* and a *backend-aware
mismatch taxonomy* so that comparison evidence stays unambiguous as backends are
added. It does not add a new wire schema, does not change default receipts, and
does not promote AST facts onto any public surface. It is a governance contract
layered over the artifacts already specified in `docs/specs/ast-shadow.md` and
`docs/specs/syntax-receipts.md`.

The shadow-only claim boundary from `docs/adr/0008-ast-foundation.md` continues
to hold for every backend identity:

- default `tokmd analyze`, `tokmd cockpit`, `tokmd context`, `tokmd handoff`,
  FFI, Python, Node, and WASM outputs must remain unchanged;
- parser-backed backends stay behind the explicit `ast` feature;
- backend identity is descriptive metadata, never a merge verdict, proof
  promotion signal, or browser capability claim;
- a backend identity does not assert correctness, reachability, undefined
  behavior, or bug presence; it only records *which* backend produced a fact.

### Claim boundary for `adze-proposed`

`adze-proposed` is a reserved identifier, not a capability claim.

- The current `ROADMAP.md` names only Tree-sitter AST work (v3.0, shadow). It
  does **not** commit to an "Adze" AST backend.
- No code emits `adze-proposed`, and no accepted roadmap item, ADR, or proposal
  in this repo defines an Adze parser backend. (The token "adze" elsewhere in
  the repo refers to an unrelated sibling repository in
  `agents/shared/droid-migration.md`, not a tokmd parser.)
- The identifier is reserved here only so that a future parser backend cannot
  silently reuse an ambiguous identity, and so reviewers have a stable name to
  reject or accept against. Introducing an actual second parser backend requires
  a fresh proposal that names its product surface, schema impact, fallback
  behavior, proof ownership, and rollback story before any code emits the
  identity.

## Inputs

A backend identity is assigned by whichever component produced a fact, using
inputs already available in the shadow and syntax paths:

- the `kind` discriminator in `tokmd.ast_shadow.v1` artifacts (`heuristic`,
  `ast`, `diff`), where the `ast` source maps to `tree-sitter` today;
- the per-language `parser_crate` recorded in `tokmd.syntax_receipt.v1` and in
  the shadow `ast.json` capabilities block (a `tree-sitter-*` crate today);
- the `parser_status` value (`parser_backed_shadow` or `unsupported`) from
  `crates/tokmd-analysis/src/ast/capability.rs`.

Backend identity assignment must not require network access, runtime parser
downloads, GitHub Actions metadata, Codecov upload, or browser/WASM support,
matching the constraints already specified for the underlying artifacts.

## Outputs

This spec governs identity *meaning*; it does not by itself add fields to the
existing artifacts.

Current mapping from existing wire values to backend identity:

| Existing wire value | Backend identity |
| --- | --- |
| `tokmd.ast_shadow.v1` `kind: "heuristic"` | `heuristic` |
| `tokmd.ast_shadow.v1` `kind: "ast"` (with a `tree-sitter-*` `parser_crate`) | `tree-sitter` |
| `tokmd.syntax_receipt.v1` `parser_crate: "tree-sitter-*"` | `tree-sitter` |

### Backend-aware mismatch taxonomy

The taxonomy unifies the comparison buckets in
`crates/tokmd-analysis/src/ast/shadow.rs` and the advisory parse statuses in
`crates/tokmd-analysis/src/ast/registry.rs` under one backend-aware vocabulary.
It is the stable language for describing a heuristic-vs-parser comparison
without claiming semantic correctness.

| Mismatch kind | Existing source value | Meaning |
| --- | --- | --- |
| `agree` | diff `matches` | both backends report the same landmark |
| `heuristic_only` | diff `heuristic_only` | landmark present for `heuristic`, absent for the parser backend |
| `backend_only` | diff `ast_only` | landmark present for the parser backend, absent for `heuristic` |
| `parse_degraded` | diff status `parse_degraded`, receipt status `parse_degraded` | parser recovered a tree but reported syntax errors (advisory) |
| `parser_failed` | receipt status `parser_failed` | parser could not load or produced no tree (advisory) |
| `unsupported` | diff status `unsupported`, receipt status `unsupported_language` / `parser_status: unsupported` | no parser backend exists for the file; only `heuristic` facts apply |
| `skipped` | receipt status `skipped_generated_or_vendor` / `skipped_too_large` | policy or size limit skipped the file (advisory) |

Every mismatch kind except `agree` is advisory. None of them is a pass/fail
verdict, proof promotion, or correctness claim. Counts must continue to match
their per-file entries, as the existing diff summary already requires.

## Compatibility

- This spec is additive documentation. It introduces no new schema family and
  changes no existing wire field, default receipt, or CLI behavior.
- The `heuristic` and `tree-sitter` identities describe behavior that already
  exists; `adze-proposed` is reserved and unimplemented.
- Adding `backend_id` as an explicit field to `tokmd.ast_shadow.v1` or
  `tokmd.syntax_receipt.v1` is a future, schema-reviewed change under
  `docs/adr/0008-ast-foundation.md`; it is intentionally out of scope here.
- Promoting any parser backend's facts onto a public surface still requires the
  function-boundary promotion criteria already specified in
  `docs/specs/ast-shadow.md`.
- Builds without `--features ast` continue to expose only the `heuristic`
  identity and must not require parser crates.

## Proof Requirements

The backend-identity and mismatch-taxonomy tables above are anchored to the
emitted wire values by
`crates/tokmd-analysis/tests/ast_shadow_backend_taxonomy.rs`: it asserts that
every `tokmd.ast_shadow.v1` and `tokmd.syntax_receipt.v1` wire value a real
producer emits maps onto exactly one documented backend identity / mismatch
kind, and that the receipt-status taxonomy is total over `SyntaxParseStatus`.
That oracle guards the documentation tables against silent drift; it adds no
`backend_id` wire field and changes no default behavior.

This spec documents identity meaning for behavior already covered by the AST
shadow proof scope. When the backend identity vocabulary, the mismatch taxonomy,
or the underlying artifact contracts change, run the `analysis_ast_shadow` scope
proofs declared in `ci/proof.toml`:

```bash
cargo test -p tokmd-analysis --features ast ast --verbose
cargo test -p xtask ast_shadow --verbose
cargo xtask ast-shadow-compare --manifest policy/ast-shadow-corpus.toml --out target/tokmd-ast-shadow --summary-md target/tokmd-ast-shadow/summary.md --timing-json target/tokmd-ast-shadow/timing.json
cargo xtask ast-shadow-check --dir target/tokmd-ast-shadow --json target/tokmd-ast-shadow/check.json
```

For a docs-only change to this spec, the shape and routing checks are sufficient:

```bash
cargo xtask doc-artifacts --check
cargo xtask docs --check
cargo xtask proof-policy --check
cargo fmt-check
git diff --check
```

If a later change introduces a real `backend_id` wire field or a second parser
backend, also update the affected schema family, capability matrix, and proof
scope, and run the relevant publish-surface checks.

## Open Questions

- Whether `backend_id` should eventually become an explicit field on
  `tokmd.ast_shadow.v1` and `tokmd.syntax_receipt.v1`, or stay a derived label
  computed from `kind` and `parser_crate`.
- Whether the mismatch taxonomy should be emitted as named string values in the
  diff artifact, or remain a documentation-level vocabulary over the existing
  buckets and statuses.
- What evidence and proposal a second parser backend must carry before any
  reserved identity (such as `adze-proposed`) is allowed to emit facts.
