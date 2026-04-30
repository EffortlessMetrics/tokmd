# Decision

## Inspected
- `xtask/tests/docs_w43.rs` checks for schema documentation sync.
- `crates/tokmd-types/src/lib.rs` contains `CONTEXT_BUNDLE_SCHEMA_VERSION`.
- `docs/SCHEMA.md` documents `CONTEXT_BUNDLE_SCHEMA_VERSION` (version 2).
- The test file `xtask/tests/docs_w43.rs` lacks a test for `CONTEXT_BUNDLE_SCHEMA_VERSION` drift, even though it tests `SCHEMA_VERSION`, `ANALYSIS_SCHEMA_VERSION`, `COCKPIT_SCHEMA_VERSION`, `CONTEXT_SCHEMA_VERSION`, and `HANDOFF_SCHEMA_VERSION`.

## Options

### Option A (recommended)
- Add a test `schema_md_context_bundle_version_matches_source` to `xtask/tests/docs_w43.rs` to assert that `CONTEXT_BUNDLE_SCHEMA_VERSION` in `crates/tokmd-types/src/lib.rs` matches the documented value in `docs/SCHEMA.md`.
- Why it fits: It closes a gap in documentation drift prevention. The test suite has similar tests for every other schema version constant, but missed `CONTEXT_BUNDLE_SCHEMA_VERSION`. By adding the missing test, we prevent silent drift for this schema constant. This squarely fits the Librarian persona's mission of improving factual docs quality and executable coverage.
- Trade-offs:
  - Structure: Improves test suite structural completeness (all schema versions checked).
  - Velocity: Negligible test execution cost.
  - Governance: Strengthens the anti-drift gating.

### Option B
- Add a learning PR explaining the missing test, but don't add the test itself.
- When to choose it: Only if there's an insurmountable reason we can't implement the test.
- Trade-offs: Leaves a clear gap in documentation verification.

## Decision
Option A. The missing test for `CONTEXT_BUNDLE_SCHEMA_VERSION` is an obvious omission in `xtask/tests/docs_w43.rs` when looking at `schema_md_context_version_matches_source`, `schema_md_handoff_version_matches_source`, etc. Adding this executable test directly serves the Librarian persona's mission to improve executable coverage to prevent silent doc drift.
