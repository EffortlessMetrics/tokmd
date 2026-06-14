## Option A (recommended)
Update the `xtask bump` schema tracker in `xtask/src/tasks/bump.rs` to include the `BASELINE_VERSION` and `SENSOR_REPORT_SCHEMA` constants. This directly targets the `tooling-governance` shard by making sure workspace release tools properly track and increment these schema versions, keeping determinism checks aligned.

## Option B
Manually check the repo or write a one-off shell script outside of xtask.
- When to choose: Never.
- Trade-offs: Increases the chance of human error and version drift during a release.

## Decision
Option A. It integrates directly into the workspace's native tooling, locks in contract determinism for the schema version tracker, and fits the Gatekeeper persona.
