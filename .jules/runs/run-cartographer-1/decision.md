## Option A (recommended)
Update `docs/ROADMAP.md` and `docs/implementation-plan.md` to reflect the completed `v1.12.0` through `v1.14.0` releases.
- **Why it fits**: Directly satisfies the Cartographer persona's target #1 (roadmap/design/requirements drift from shipped reality).
- **Trade-offs**:
  - **Structure**: High. Keeps planning documentation truthful and prevents contributors from picking up work that is already shipped.
  - **Velocity**: Low impact on product features, high impact on accurate contributor context.
  - **Governance**: High. Synchronizes the agent workbench roadmap with the actual EffortlessMetrics/tokmd release reality.

## Option B
Do not touch the planning docs and create a learning PR documenting that the docs are out of date.
- **When to choose it instead**: If the drift was too vast to fix in a single PR safely, or if we lacked access to the release ledger to verify what actually shipped.
- **Trade-offs**: Misses an easy, concrete fix that directly aligns documentation with the release ledgers.

## ✅ Decision
Option A. The release ledgers (`1.12-ledger.md`, `1.13-ledger.md`, `1.14-ledger.md`) explicitly document what shipped. Updating the `docs/ROADMAP.md` and `docs/implementation-plan.md` to include these completed phases fixes factual drift and clearly closes the previously active "PR evidence packet workflows" lane.
