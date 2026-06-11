# Decision: Schema documentation sync drift prevention

## Option A (recommended): Extend `schema_sync` test coverage
- what it is: Add tests in `crates/tokmd/tests/schema_sync.rs` to verify that the versions documented in `docs/SCHEMA.md` for `Tool`, `Baseline`, and `Envelope` match the actual values defined in code (`TOOL_SCHEMA_VERSION`, `BASELINE_VERSION`, `SENSOR_REPORT_SCHEMA`).
- why it fits this repo and shard: The `tooling-governance` shard specifically handles workspace workflows and documentation correctness. The problem explicitly mentions schema/version drift as the #1 target ranking. Extending these test expectations prevents documentation from falling out of sync with code logic.
- trade-offs: Structure / Velocity / Governance: Structure is improved by guaranteeing sync. Velocity hit is negligible since we run validation locally in milliseconds. Governance is improved by cementing the schema version source of truth.

## Option B: Update doc and test separately via CI bash validation
- what it is: Add grep scripts inside `.github/workflows` to fail CI if versions drift.
- when to choose it instead: If tests inside Rust were too complicated due to macro-expanded code logic.
- trade-offs: Harder to maintain over time, fragments validation logic.

## ✅ Decision
Option A. It's the standard practice in `tokmd` to use test-driven constraints (as done by `schema_sync.rs`).
