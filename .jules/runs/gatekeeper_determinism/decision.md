# Decision

## Option A (recommended)
Record a friction item describing the schema drift and file a learning PR. The `BASELINE_VERSION` constant was added to `crates/tokmd-analysis-types/src/baseline.rs` and documented in `docs/SCHEMA.md`, but it was omitted from the centralized `SCHEMA_LOCATIONS` array in `xtask/src/tasks/bump.rs`. This creates a risk of version drift where running `cargo xtask bump --schema` would fail to update or validate this contract. Because `xtask/src/tasks/bump.rs` is outside the `core-pipeline` shard, making the fix directly would violate shard discipline. The correct procedure is to surface the drift as friction and exit gracefully as a learning PR.

## Option B
Modify `xtask/src/tasks/bump.rs` directly. This directly fixes the schema version drift. However, it violates prompt constraints and risks corrupting tools outside the assigned domain.

## Decision
**Option A**. Sticking to the assigned shard is a hard constraint. The friction item properly documents the gap so a future run assigned to `xtask` can safely apply the fix.
