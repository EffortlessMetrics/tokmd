Option A (recommended):
- what it is: Update `docs/SCHEMA.md`, `docs/design.md`, `docs/architecture.md`, `docs/agent-context/review-invariants.md`, and `docs/specification.md` to properly document the new `TOOL_SCHEMA_VERSION`. It is missing from `SCHEMA.md` entirely.
- why it fits this repo and shard: It fits the Librarian persona which focuses on correcting drift between docs and implementation, specifically in reference docs like `SCHEMA.md` and schema files. The shard `tooling-governance` clearly covers `docs/**`.
- trade-offs: Structure / Velocity / Governance: Improves governance and documentation structure with minimal impact on velocity.

Option B:
- what it is: Do not document `TOOL_SCHEMA_VERSION` in the schema/design markdown files.
- when to choose it instead: If `TOOL_SCHEMA_VERSION` is an internal implementation detail that shouldn't be publicly documented as part of the schema versioning system.
- trade-offs: Might lead to confusion about schema versioning since `TOOL_SCHEMA_VERSION` is explicitly listed in `xtask/src/cli.rs` and `xtask/src/tasks/bump.rs` alongside all other documented schemas.

Decision: I will choose Option A because `TOOL_SCHEMA_VERSION` clearly has a defined constant `pub const TOOL_SCHEMA_VERSION: u32 = 1;` in `crates/tokmd/src/tool_schema.rs` and the `xtask bump` documentation explicitly mentions it alongside `SCHEMA_VERSION`, `ANALYSIS_SCHEMA_VERSION`, etc. It seems to have been missed in the `SCHEMA.md` documentation during development.
