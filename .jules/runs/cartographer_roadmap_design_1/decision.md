# Cartographer Decision: ROADMAP & Docs Alignment with AST Reality

## Context
The prompt requires finding and fixing factual drift between shipped reality and roadmap/design/requirements docs, specifically optimizing for "useful, aligned, evidence-backed work per prompt" and treating it as a Rust-first shard.

Upon analyzing the documentation and codebase:
1. `ROADMAP.md` and `docs/implementation-plan.md` still state that Tree-sitter AST parsing is part of the v3.0 / Phase 7 "Long-term" goals and explicitly assert that "Default full AST analysis — tokmd remains heuristic-first until feature-gated AST shadow evidence justifies public receipt/schema changes" (and similarly throughout the docs).
2. Furthermore, in `ROADMAP.md`, `v4.0.0` is designated for `Adze AST integration.`
3. Meanwhile, `crates/tokmd-analysis/src/ast/rust.rs` already contains a partial implementation of Tree-sitter integration behind the `ast` feature flag, as defined in `ADR-0008` (which is marked as "proposed", yet the code is actively merged and feature-gated in `Cargo.toml`).

The drift here isn't that AST is fully rolled out, but rather that the AST foundation (ADR-0008) is partially shipped in code (`crates/tokmd-analysis/Cargo.toml` has `ast = [...]`, `src/ast/rust.rs` uses `tree-sitter`, and `NEXT.md` acknowledges "The Rust AST shadow scaffold now uses optional `tree-sitter`"). However, `ROADMAP.md` under "Completed Milestones" fails to mention this work landing in the recent v1.x milestones, despite `NEXT.md` stating "The Rust AST shadow scaffold now uses optional tree-sitter".

Wait, `ROADMAP.md` says:
```markdown
| **v3.0.0** | 🔭 Long-term | Tree-sitter AST integration (requires significant R&D).      |
| **v4.0.0** | 🔭 Long-term | Adze AST integration.      |
```
And under Phase 7 in `docs/implementation-plan.md`:
```markdown
- [x] Evaluate Tree-sitter Rust grammar availability and dependency footprint for a first shadow slice
- [x] Add feature-gated `tokmd-analysis::ast` owner module
- [x] Parse initial Rust function landmarks
```
So `docs/implementation-plan.md` IS up to date with the checkboxes!
But `ROADMAP.md`'s Status Summary and Completed Milestones don't mention that the feature-gated shadow scaffold landed, which is significant R&D that is already in tree.

Let's look for a better target: "roadmap/design/requirements drift from shipped reality" or "missing explanation of a real architectural/design choice" or "stale implementation-plan sections that mislead contributors".

Let's check `ROADMAP.md` Completed Milestones for v1.11.0:
```markdown
| **v1.11.0** | ✅ Complete | Browser runtime polish: explicit cache behavior, progress events, retry/rate-limit UX, and authenticated fetch. |
```
Is `tokmd cockpit` review-packet hardening missing from ROADMAP?
`NEXT.md` says:
```
The active product lane is cockpit review-packet hardening. Improve cockpit as the PR-review evidence surface before adding any separate `tokmd review` command. After the review packet/action surface is stable, move into the architecture-consolidation program.
```
In `ROADMAP.md`, `v1.11.0` is complete. What is next in ROADMAP?
```markdown
## Future Horizons

### v2.0 — Platform Evolution

#### A. AI Agent Integration & MCP Server Mode
```

What about architecture consolidation or review packet hardening? They are missing from `ROADMAP.md` entirely. In `ROADMAP.md`, there's no mention of the architecture consolidation program or cockpit review-packet hardening in the Future Horizons.

Let's review `docs/implementation-plan.md`.

Looking at `NEXT.md` vs `docs/implementation-plan.md` and `ROADMAP.md`:
`NEXT.md` specifies: "The active product lane is cockpit review-packet hardening. Improve cockpit as the PR-review evidence surface before adding any separate `tokmd review` command. After the review packet/action surface is stable, move into the architecture-consolidation program."

However, `ROADMAP.md` completely misses the "cockpit review-packet hardening" and "architecture-consolidation" phases, jumping straight to "v2.0 MCP Server Mode". `docs/implementation-plan.md` similarly has Phase 5c (Browser Runtime Polish) and then Phase 6 (MCP Server Mode), completely skipping the active product lane and architecture-consolidation program mentioned in `NEXT.md`.

In `docs/implementation-plan.md`, Phase 5c (v1.11.0) is marked as complete in `ROADMAP.md` but not in `implementation-plan.md` header (it lacks the `✅ Complete` badge).
Wait, `NEXT.md` says: "The v1.11 browser runtime polish lane is closed on main: cache semantics, worker and repo-load progress, retry/rate-limit guidance, authenticated fetch UX, loaded-bundle capability filtering, and local browser file input are all implemented."

So `docs/implementation-plan.md` is slightly stale on v1.11.0 being complete, and both `ROADMAP.md` and `docs/implementation-plan.md` are missing the current active product lanes described in `NEXT.md`:
1. Cockpit review-packet hardening (improve cockpit as PR-review evidence surface).
2. Architecture consolidation program (collapsing implementation microcrates into SRP modules).

Let's check `docs/implementation-plan.md` for "architecture-consolidation" or "cockpit".

So the factual drift between shipped reality / NEXT directives and the roadmap/design docs is:
1. `docs/implementation-plan.md` still lists `Phase 5c: Browser Runtime Polish (v1.11.0)` as incomplete (missing `✅ Complete` in header), even though it is closed on main according to `NEXT.md` and `ROADMAP.md` Completed Milestones.
2. `ROADMAP.md` and `docs/implementation-plan.md` both skip the current active program ("cockpit review-packet hardening" and "architecture-consolidation") mandated by `NEXT.md`.
3. `ROADMAP.md` does not list the architecture consolidation or cockpit review-packet hardening under `Future Horizons`. It just jumps to v2.0 Platform Evolution.
4. Also `ROADMAP.md` has `| **v1.11.0** | ✅ Complete | Browser runtime polish...` but it's not marked `✅ Complete` in the implementation-plan phase heading.

Let's fix this factual drift.

### Option A (recommended)
Update `docs/implementation-plan.md` and `ROADMAP.md` to reflect the active product lane:
- Mark `Phase 5c: Browser Runtime Polish (v1.11.0)` as `✅ Complete` in `docs/implementation-plan.md`.
- Insert a new section in `ROADMAP.md` (e.g. `v1.12.0 — Cockpit Review-Packet Hardening & Architecture Consolidation`) under `Future Horizons` before `v2.0` to reflect the directives in `NEXT.md`.
- Insert a new Phase (e.g., `Phase 5d` or `Phase 6`) in `docs/implementation-plan.md` for `Cockpit Review-Packet Hardening` and `Architecture Consolidation` to bridge the gap between v1.11 and v2.0, aligning it with `NEXT.md`.

This aligns the roadmap and implementation plan with the real architectural and design choice mandated by `NEXT.md` (which represents the shipped/active reality of the project's direction).

### Option B
Only update `ROADMAP.md` to mention the active product lane, leaving `docs/implementation-plan.md` alone. This is inferior because `docs/implementation-plan.md` is also explicitly used by contributors and contains the stale `Phase 5c` and lacks the active product phases.

### Decision
I will proceed with Option A to fully align `ROADMAP.md` and `docs/implementation-plan.md` with the ground truth in `NEXT.md` and the shipped v1.11.0 reality.
