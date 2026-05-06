# Decision

## Option A (recommended)
Update `docs/requirements.md` to fix factual drift regarding `Analysis receipts` schema version. It currently states "Schema v5" while `docs/design.md`, `docs/architecture.md`, and the codebase itself (`crates/tokmd-analysis-types/src/lib.rs`) state it's `ANALYSIS_SCHEMA_VERSION = 9`.

- **What it is**: Updating the `Receipt Contracts` section in `docs/requirements.md` to list `Analysis receipts` as `Schema v9` instead of `Schema v5`.
- **Why it fits this repo and shard**: Aligning documentation with the actual implementation is the Cartographer persona's primary goal. The "tooling-governance" shard covers `docs/**`, and fixing this schema version inconsistency is directly in scope.
- **Trade-offs**:
  - **Structure**: High alignment. It removes contradictory schema claims across design docs.
  - **Velocity**: Fast to execute.
  - **Governance**: Ensures requirements docs aren't misleading developers about contract versions.

## Option B
Update `docs/requirements.md` to just point to `docs/design.md` for schema versions.

- **What it is**: Removing the specific schema version numbers from `docs/requirements.md` and adding a link to `docs/design.md` where the definitive list is kept.
- **When to choose it instead**: If we want a single source of truth for schema versions to avoid future drift.
- **Trade-offs**:
  - Decreases the immediate readability of the requirements document. It's often useful to state current expectations in requirements.

## Decision
I will proceed with **Option A**. The requirements document outlines what the system *must* do, and stating the current schema version there is standard practice. Updating it to `v9` fixes the immediate factual drift while keeping the document informative.
