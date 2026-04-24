# Decision

## Inspected
- `.jules/docs/` containing legacy ledger and run packets
- `.jules/quality/` containing legacy ledger and run packets
- `.jules/bin/build_index.py` which only looks at `.jules/runs/`

## Options Considered

### Option A (Recommended)
Migrate legacy ledgers and run documents from `.jules/docs/` and `.jules/quality/` into the central `.jules/runs/<run-id>` format and delete the old ledger locations.
- **Why it fits:** The Archivist persona mission is to "improve Jules itself by consolidating run packets, friction, learnings, and shared scaffolding." The memory explicitly says: "Legacy ledgers in .jules/docs/ or .jules/quality/ are deprecated."
- **Trade-offs:**
  - Structure: Improves consistency, ensures `build_index.py` captures all history.
  - Velocity: Low immediate product velocity impact, but speeds up future run metadata checks.
  - Governance: Unifies source of truth for run artifacts.

### Option B
Only update `.jules/bin/build_index.py` to parse the old folders as well.
- **Why it fits:** Requires no migration of existing data.
- **Trade-offs:**
  - Structure: Keeps technical debt and fragmented formats around permanently.

## Decision
Chose **Option A**. The system explicitly deprecates the old folders and expects a standardized format per run in `.jules/runs/<run-id>`. Migrating them properly fulfills the Archivist persona's mission to consolidate run packets.