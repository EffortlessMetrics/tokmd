## Target Selection
Migrating legacy ledgers to unified run packets.

## Option A (recommended)
Migrate the legacy `.jules/docs/` and `.jules/quality/` ledgers and envelopes into the unified `.jules/runs/` packet format, then delete the legacy directories and update the rollup index.
- **Why it fits**: Consolidates run history into the single source of truth format defined in `RUN_PACKET.md`, keeping the index builder script working.
- **Trade-offs**:
  - Structure: Improves consistency and simplifies `.jules` layout.
  - Velocity: Future Jules agents don't have to parse multiple legacy formats.
  - Governance: Maintains historical data without deleting any past receipts.

## Option B
Retain `.jules/docs/` and `.jules/quality/` as separate ledgers and update the indexing script to parse all formats.
- **When to choose**: If we have external processes relying on the exact legacy file layouts and formats.
- **Trade-offs**:
  - Increases complexity of indexing and run history parsing, violating the "Make truth cheap" principle.

## Decision
Chose **Option A**. The single unified `.jules/runs/` directory is explicitly required by `RUN_PACKET.md` ("Each run writes a self-contained packet under: `.jules/runs/<run-id>/`"). Migrating the old envelopes simplifies the repository and aligns all historical data.
