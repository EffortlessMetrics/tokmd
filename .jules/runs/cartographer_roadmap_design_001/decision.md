# Decision

## Option A (recommended)
Update `docs/implementation-plan.md` to reflect that Phase 5 (WASM-Ready Core + Browser Runner) shipped in `v1.9.0`. This includes adding the `✅ Complete` badge and checking the work item boxes. Additionally, fix the stale reference under Phase 7 (Tree-sitter) that incorrectly lists MCP server mode as "Phase 5" instead of "Phase 6".

**Why it fits this repo and this shard**:
As the Cartographer persona, the goal is to align design/implementation docs with the shipped reality. `ROADMAP.md` and `NOW.md` correctly reflect that `v1.9.0` (WASM productization) shipped, but `docs/implementation-plan.md` still shows Phase 5 as incomplete with unticked checkboxes. This fixes a factual drift.

**Trade-offs**:
- Structure: Improves doc consistency across the repo.
- Velocity: Very fast, low-risk update.
- Governance: Reduces confusion for future contributors reading the implementation plan.

## Option B
Do not touch `docs/implementation-plan.md` and instead write a learning PR noting the drift.

**When to choose it instead**:
If the drift was intentional or if we lacked proof that `v1.9.0` actually shipped with those features.

**Trade-offs**:
- Misses an easy, aligned win to fix real factual drift in the core planning documentation.

## Decision
**Option A**. The repo's `ROADMAP.md` and `docs/NOW.md` explicitly state `v1.9.0` shipped with the WASM runner features. The implementation plan was simply left behind during the release process. I will update `docs/implementation-plan.md` to check off Phase 5 and fix the incorrect Phase 6 reference.
