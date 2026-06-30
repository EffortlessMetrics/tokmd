# Plan: AST / Syntax Productization

- Status: active
- Related proposal: `docs/proposals/ast-productization.md`
- Related spec:
  `docs/specs/syntax-receipts.md`,
  `docs/specs/ast-shadow.md`,
  `docs/specs/ast-shadow-backend.md`
- Related ADR: `docs/adr/0008-ast-foundation.md`

## Goal

Reconcile governance with the shipped AST/syntax surface and close remaining
correctness and documentation gaps so explicit opt-in syntax evidence is
discoverable, honest, and CI-proven—without promoting AST facts onto default
public receipts.

## Non-goals

- Do not bump receipt schema versions for AST-derived default output.
- Do not make `tokmd analyze` run syntax parsing by default.
- Do not add browser/WASM tree-sitter in this plan.
- Do not publish crates, tag releases, or move release aliases.
- Do not overturn the function-boundary candidate `not yet` outcome for public
  receipt promotion.

## Work Packets

1. **CLI correctness — `tokmd syntax --exclude`** (PR #368, branch
   `fix/syntax-honor-exclude`)
   - Honor global `--exclude` when collecting syntax paths.
   - Proof: `cli_syntax_integration`, no-panic strict, clippy, fmt.
2. **Governance reconciliation** (this plan's PR on branch
   `ast-productization-lane`)
   - Accept `docs/proposals/ast-productization.md`.
   - Update `NEXT.md`, `ROADMAP.md`, `NOW.md`, `ast-shadow.md`, README,
     `reference-cli.md`, `ci/proof.toml` routing.
3. **Optional: packet exclude forwarding**
   - Pass global/packet excludes into `write_syntax` when generating packet
     `syntax.json`.
4. **Optional: user-path syntax guide**
   - Add a recipes or workflows section for UB/crash review using
     `review_signals` (no new command).
5. **Publication import checkpoint**
   - True merge-commit import after packets 1–2 merge; FF swarm; repo-graph
     aligned.

## Validation

```bash
cargo test -p tokmd-analysis --features ast ast --verbose
cargo test -p xtask ast_shadow --verbose
cargo test -p tokmd --features ast --test cli_syntax_integration --verbose
cargo xtask ast-shadow-compare --manifest policy/ast-shadow-corpus.toml --out target/tokmd-ast-shadow --summary-md target/tokmd-ast-shadow/summary.md --timing-json target/tokmd-ast-shadow/timing.json
cargo xtask ast-shadow-check --dir target/tokmd-ast-shadow --json target/tokmd-ast-shadow/check.json
cargo xtask doc-artifacts --check
cargo xtask docs --check
cargo xtask proof-policy --check
cargo fmt-check
```

Required CI: `Tokmd Rust Result` (`cargo test --all-features`).

## Stop Conditions

- Stop if a change requires default receipt or public schema promotion—open a
  fresh schema-reviewed proposal instead.
- Stop if function-boundary candidate criteria are not met but the PR tries to
  wire AST into `analyze` or cockpit receipts.
- Do not version-bump or release solely because this plan's governance packets
  complete.

## Outcome tracking

| # | PR | Status |
| --- | --- | --- |
| 1 | #368 syntax `--exclude` | merged |
| 2 | #369 governance reconciliation | merged |
| 3 | #370 packet exclude forwarding | merged |
| 4 | user-path syntax guide | in progress |
| 5 | publication import | after lane packets |

## Intentionally deferred

- Default `analyze` syntax enrichment
- Public schema fields for AST-derived metrics
- Browser/WASM tree-sitter
- Function-boundary public candidate (still `not yet`)
