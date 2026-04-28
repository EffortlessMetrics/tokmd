# Decision

## Option A (recommended)
Update `docs/implementation-plan.md` to mark Phase 5 (WASM & Browser Runner v1.9.0) work items as completed (`[x]`). Additionally, fix the stale references in Phase 7 (Tree-sitter) that incorrectly refer to "MCP server mode (Phase 5)" instead of "Phase 6".
- Fits the repo/shard because the `Cartographer` persona requires fixing factual drift between shipped reality and roadmap/design docs. `ROADMAP.md` explicitly lists v1.9.0 as shipped, but `docs/implementation-plan.md` has it marked as incomplete.

## Option B
Create a learning PR documenting this discrepancy without fixing it.
- Choose if we aren't confident about which parts shipped. Since `ROADMAP.md` is explicit about what shipped in v1.9.0 (all 5 items match the work items), we can safely fix it instead.

## Decision
Option A. It's an honest patch that directly aligns the docs with the shipped reality described in `ROADMAP.md` and `docs/architecture.md`.
