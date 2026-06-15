# Cartographer Decision

## Target identification
Through exploring the codebase, I see the following drift targets:
1. `ROADMAP.md` is out of date. It tracks up to `v1.11.0` and marks `v2.0.0` as planned. However, the current package version is `1.13.1`. The roadmap completely misses `1.12.0` (Bun UB evidence readiness and `tokmd-swarm` workbench) and `1.13.0` (Syntax-aware evidence packet release). It also lists "v1.12.x — Selection-First Product and Evidence Work" under a future section.
2. `ROADMAP.md` still refers to AST as "Shadow-only", but `1.13.0` shipped `tokmd syntax` behind the `ast` feature, and `1.13.1` made `ast` included by default in the published `tokmd` crate.

## Options considered
### Option A (recommended)
- what it is: Update `ROADMAP.md` to record `v1.12` and `v1.13` as shipped reality. Add them to the "Status Summary" table and "Completed Milestones". Remove the stale "v1.12.x" section and update the AST status.
- why it fits this repo and shard: Fixes factual drift between shipped reality and roadmap/design docs, aligning perfectly with Cartographer's mission.
- trade-offs: Structure: Keeps the document chronologically accurate. Velocity: Helps reviewers understand what is actually shipped vs planned. Governance: Accurate historical tracking.

### Option B
- what it is: Fix missing schema constants in `xtask/src/tasks/bump.rs` like `BASELINE_VERSION` and `SENSOR_REPORT_SCHEMA`.
- when to choose it instead: If playing Gatekeeper/Steward to fix release tooling.
- trade-offs: Not the primary mission of Cartographer, and `cargo xtask version-consistency` currently passes. I will create a friction item for it instead.

## Decision
I will proceed with Option A and create a friction item for Option B.
