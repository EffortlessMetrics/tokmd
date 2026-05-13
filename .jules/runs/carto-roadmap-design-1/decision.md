# Cartographer Decision: Factual Drift in Roadmap/Implementation docs

## Option A (recommended)
- **What it is**: Update the roadmap/implementation docs to correctly reflect the current state of AST integration. `docs/implementation-plan.md` lists Phase 7 as "(v3.0 - Shadow Mode Active)", while `ROADMAP.md` still lists `v3.0` as "Tree-sitter Integration (Long-term)" deferring it "well beyond the v2.x roadmap". Furthermore, `ROADMAP.md` indicates `v1.12.0` is "Active", but the changelog shows `v1.11.0` was released on 2026-05-08 and we're past that.
- **Why it fits**: The prompt specifically asks to fix factual drift between shipped reality and roadmap/design/requirements docs. The codebase has shipped the `ast` feature in `crates/tokmd-analysis` behind ADR-0008, so "Shadow Mode Active" is real, but the Roadmap still says "Long-term" and "deferred well beyond v2.x".
- **Trade-offs**:
  - *Structure*: Corrects outdated timeframes in long-term goals without altering the actual technical plans.
  - *Velocity*: Fast, high confidence.
  - *Governance*: Aligns the public-facing ROADMAP with ADR-0008 and `implementation-plan.md`.

## Option B
- **What it is**: Create a friction item because `ROADMAP.md` says v1.12 is Active, but the changelog says v1.11 is the latest, and maybe there's no code fix needed.
- **Trade-offs**: Fails to actually correct the "AST integration" discrepancy between the ROADMAP, Implementation Plan, and shipped code (which already has the feature-gated `ast` mod).

## Decision
Choose Option A. Update `ROADMAP.md` to show that the Tree-sitter integration shadow mode is actively running, rather than being "deferred well beyond the v2.x roadmap". `ROADMAP.md` should match the reality described in `docs/implementation-plan.md` and `docs/adr/0008-ast-foundation.md`.
