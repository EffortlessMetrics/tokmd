## Problem
The `docs/implementation-plan.md` has stale sections that mislead contributors.
Specifically, `Phase 3: tokmd-core Stabilization` is outdated.
It lists:
- "Define port traits" as incomplete.
- "Publish tokmd-core to crates.io (when stable)" as incomplete.

However:
1. `tokmd-io-port` was landed in Phase 4d, so port traits exist. The "Define port traits" task is conceptually complete (even though they are separated into `tokmd-io-port`).
2. `tokmd-core` has already been published to crates.io (it's marked as public product in `docs/architecture-consolidation-plan.md` and `Phase 3` is structurally behind `Phase 4d`). Actually, Phase 3 is NOT marked as "✅ Complete" in `implementation-plan.md` but all subsequent phases are!

This is a clear target for "stale implementation-plan sections that mislead contributors" and "roadmap/design/requirements drift from shipped reality" as specified by the persona instructions. We should update `Phase 3` to be `✅ Complete` and tick the remaining boxes based on the current reality of the system.

## Option A (recommended)
Update `Phase 3: tokmd-core Stabilization` to be `✅ Complete`.
- Check off "Define port traits" (since `tokmd-io-port` provides this).
- Check off "Add comprehensive API documentation" (core lib is well-documented).
- Check off "Publish tokmd-core to crates.io" (it is published).
- Mark Phase 3 as complete in the heading.

**Why it fits:** Directly addresses roadmap/implementation-plan drift, matching the #3 target in Cartographer's ranking.
**Trade-offs:**
- *Structure*: Keeps the implementation plan accurate to shipped reality.
- *Velocity*: Eliminates contributor confusion about whether the core API is stable/published.
- *Governance*: Aligns docs with actual published crate status.

## Option B
Delete Phase 3 entirely.
**When to choose:** If the phase was abandoned.
**Trade-offs:** We lose the historical record of the work, which goes against the document's purpose as a record.

## Decision
Proceed with **Option A**. The system *did* stabilize tokmd-core, and the `tokmd-io-port` crate fulfilled the port requirements.
