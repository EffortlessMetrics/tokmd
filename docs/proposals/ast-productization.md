# Proposal: AST / Syntax Productization Lane

- Status: accepted
- Owner: analysis_ast_shadow
- Related issues: user-directed AST lane opening (2026-06-30)
- Related specs:
  `docs/specs/ast-shadow.md`,
  `docs/specs/ast-shadow-backend.md`,
  `docs/specs/syntax-receipts.md`
- Related ADRs: `docs/adr/0008-ast-foundation.md`
- Related plans: `docs/plans/ast-productization.md`

## Problem

Governance docs (`docs/NEXT.md`, `docs/specs/ast-shadow.md`, ROADMAP Lane 6)
still record **"no active AST productization lane"** and treat AST as
shadow-only developer evidence. Ground-truth scouting (2026-06-30) shows the
implementation has already passed that stage for explicit opt-in surfaces:

| Surface | State | Proof |
| --- | --- | --- |
| `tokmd syntax` CLI | shipped (default `ast` feature) | `cli_syntax_integration.rs`, `reference-cli.md` |
| `tokmd packet generate --syntax` | wired | `packet_generate_integration.rs` |
| 4-language tree-sitter registry | Rust/TS/TSX/Python | `registry.rs` tests + syntax fixtures |
| AST shadow compare/check | developer tooling | `cargo xtask ast-shadow-compare/check` |
| Backend identity taxonomy | documented + oracle | `ast_shadow_backend_taxonomy.rs` |

The gap is **governance drift**, not missing parser code. Maintainers and agents
reading `NEXT.md` would defer work that is already done or block fixes (e.g.
`tokmd syntax` ignoring `--exclude`) because the lane looked closed.

Prior function-boundary candidate decisions remain **`not yet`** for *public
receipt promotion* — that is a separate question from whether the explicit syntax
product surface is working.

## Goals

Define what **"fully working and productized"** means for tokmd AST/syntax in
this repo, reconcile source-of-truth docs with shipped behavior, and sequence
remaining narrow PRs without default-receipt or schema promotion.

### Productized (this lane owns)

1. **Explicit syntax command** — `tokmd syntax <paths>` emits
   `tokmd.syntax_receipts.v1` with per-file `tokmd.syntax_receipt.v1`,
   advisory review signals, panic-seam summaries (Rust), and explicit
   degradation statuses.
2. **Packet integration** — `tokmd packet generate --syntax` writes
   `syntax.json` into evidence packets when the `ast` feature is enabled.
3. **Shadow evidence tooling** — `cargo xtask ast-shadow-compare` and
   `cargo xtask ast-shadow-check` over `policy/ast-shadow-corpus.toml`.
4. **CI proof** — `analysis_ast_shadow` scope + `cargo test --all-features`
   (required `Tokmd Rust Result`) exercise the above.
5. **Documentation** — CLI reference, README quick-path, artifact glossary
   entries, and honest capability boundaries.

### Still shadow-only (out of scope without fresh schema review)

- Default `tokmd analyze`, `cockpit`, `context`, `handoff` receipt fields
- FFI / Python / Node / WASM syntax or AST capability
- Browser tree-sitter bundles
- Public promotion of function-boundary precision onto analysis receipts
- Merge verdicts or proof-promotion claims from syntax evidence

## Non-goals

- Do not bump `SCHEMA_VERSION`, `ANALYSIS_SCHEMA_VERSION`, or receipt families
  for AST-derived facts in this lane.
- Do not make `tokmd analyze` run syntax parsing by default.
- Do not publish crates, tag releases, or move release aliases.
- Do not add new tree-sitter grammars without registry spec review.
- Do not resurrect the `adze-proposed` backend identity as implemented code.

## Options

### Option A — Reconcile docs only; keep `ast` in default features (recommended)

Treat the syntax surface as **opt-in at command level, included in default
binary builds**. Update governance docs, fix remaining CLI correctness gaps,
and defer default-receipt promotion until function-boundary evidence clears the
existing candidate criteria.

**Pros:** Matches shipped behavior; no breaking build change; honest support
tier ("explicit command, advisory receipts").

**Cons:** Default binary carries tree-sitter dependency weight (already true).

### Option B — Remove `ast` from default features

Make syntax require `cargo build --features ast` or a dedicated distribution
profile.

**Pros:** Smaller default dependency footprint.

**Cons:** Breaking change for consumers expecting `tokmd syntax` in release
binaries; CI and packet workflows already assume `ast` on main.

### Option C — Promote syntax into default `analyze` output

Add optional syntax section to analysis receipts.

**Rejected for this lane:** requires schema bump, fallback design, and
function-boundary candidate decision still `not yet`.

## Recommendation

**Adopt Option A.** Declare the AST/syntax lane **active** with this support
tier:

| Capability | Support tier | User entry |
| --- | --- | --- |
| `tokmd syntax` | experimental, opt-in command | `tokmd syntax <paths>` |
| Packet `syntax.json` | experimental, `--syntax` flag | `tokmd packet generate --syntax` |
| AST shadow compare/check | developer tooling | `cargo xtask ast-shadow-compare` |
| Default analyze/cockpit receipts | unchanged (heuristic) | existing commands |

**Immediate PR sequence (after this proposal):**

1. **CLI correctness** — `tokmd syntax` honors global `--exclude` (PR #368).
2. **Governance reconciliation** — this proposal + `NEXT.md` / `ROADMAP` /
   spec stale-text cleanup (this PR).
3. **Packet exclude forwarding** (optional follow-on) — pass packet/global
   excludes into syntax generation when scoped.
4. **User-path doc** — short recipes/workflows section for crash-hunting / UB
   review using `review_signals` (optional follow-on PR).
5. **Publication import** — merge-commit batch after PRs 1–2 land.

**Deferred until fresh evidence + schema review:**

- Function-boundary promotion onto public analysis receipts
- Browser/WASM AST feasibility implementation
- Explicit `backend_id` wire field (spec open question;
  derived labels suffice today)

## Open Questions

1. **Default feature policy:** keep `ast` in `tokmd` default features, or split
   a `tokmd-minimal` profile? (Recommendation: keep; document opt-in command
   boundary.)
2. **Analyze preset:** should a future `syntax` or `bun-ub` preset call
   `tokmd syntax` internally, or stay packet-only?
3. **Corpus expansion:** extend `policy/ast-shadow-corpus.toml` beyond Rust-only
   shadow comparison for TS/Python syntax receipts?
4. **Release:** when is AST syntax surface stable enough for release-notes
   promotion from experimental to supported? (Not this lane.)
