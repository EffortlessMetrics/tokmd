# Decision

## Option A (recommended)
Update `.jules/bin/build_index.py` to aggregate run history natively from existing ledgers (`.jules/docs/ledger.json` and `.jules/quality/ledger.json`) in addition to `.jules/runs/`.
- **Why it fits:** Preserves historical ledgers as primary truth instead of rewriting history, matching the `Archivist` proof expectations and instructions to summarize/supersede ledgers natively.
- **Trade-offs:**
  - Structure: Centralizes index generation.
  - Velocity: Immediate visibility of old and new run formats.
  - Governance: Stops rewriting history while maintaining complete auditability.

## Option B
Write a script to migrate all historical ledger entries into `.jules/runs/<run-id>/` packets.
- **When to choose it instead:** If historical formats were explicitly deprecated and required to be destroyed.
- **Trade-offs:** Violates the directive to never delete or rewrite historical ledger data already tracked in version control.

## Decision
Selected **Option A**. Updating `build_index.py` consolidates run metrics securely without altering or erasing source-of-truth ledgers.
