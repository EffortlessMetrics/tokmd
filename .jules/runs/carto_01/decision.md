# Option A (recommended)
Update `docs/implementation-plan.md` to mark Phase 5 as complete.
- **What it is**: Updating the document `docs/implementation-plan.md` to reflect the shipped reality where Phase 5 (v1.9.0) WASM-Ready Core + Browser Runner is complete. This aligns the `implementation-plan.md` with `ROADMAP.md` which already documents v1.9.0 as fully shipped, and `CHANGELOG.md` which includes v1.9.0.
- **Why it fits this repo and shard**: The primary goal of Cartographer is to fix design/roadmap drift. The `ROADMAP.md` has v1.9.0 checked off and completed, but the implementation plan still listed the items as uncompleted.
- **Trade-offs**: Structure / Velocity / Governance: Aligns documentation with the shipped state, making future planning and contribution cleaner.

# Option B
Update `ROADMAP.md` instead to say Phase 5 is still active.
- **What it is**: Reverting `ROADMAP.md` to reflect that Phase 5 is still in progress.
- **When to choose it instead**: If the features are actually missing from the codebase.
- **Trade-offs**: Fails to acknowledge the current v1.9.0 shipped state.

# Decision
We choose Option A because the code has actually shipped v1.9.0 as documented in `ROADMAP.md` and `CHANGELOG.md`, but `docs/implementation-plan.md` was out of date, listing v1.9.0 tasks as incomplete.
