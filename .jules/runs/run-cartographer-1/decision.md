# Decision: Fix Architecture Drift

## Target
The `ROADMAP.md` doc contains an outdated `Crate Hierarchy` table that does not match the actual crate hierarchy represented in `docs/architecture.md` and the workspace package configuration.

## Options considered

### Option A: Fix ROADMAP.md to align with architecture.md and publish list
- **What it is:** Update the `Crate Hierarchy` table in `ROADMAP.md` to reflect the current state (e.g. adding `tokmd-io-port` to Tier 0, `tokmd-cockpit` to Tier 3, fixing missing Tier 5, syncing Tier descriptions).
- **Why it fits:** The prompt explicitly asks to look for "roadmap/design/requirements drift from shipped reality" and "missing explanation of a real architectural/design choice" under the `tooling-governance` shard. `ROADMAP.md` has drifted from `architecture.md` and reality.
- **Trade-offs:**
  - *Structure:* Better alignment with the source of truth (`docs/architecture.md`).
  - *Velocity:* High, quick fix.
  - *Governance:* Reduces confusion for contributors.

### Option B: Rewrite the whole ROADMAP.md
- **What it is:** Fully restructure the roadmap based on current active goals and architecture.
- **Why it fits:** It's more comprehensive.
- **Trade-offs:**
  - High effort, likely out of scope for a single PR change.

## Decision
**Option A**. Updating the `ROADMAP.md` table to match the current crates.io publish reality and `docs/architecture.md` fixes a concrete drift issue cleanly within the scope and shard.
