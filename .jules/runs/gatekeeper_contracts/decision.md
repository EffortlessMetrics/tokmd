# Decision

## Target
The baseline complexity schema `docs/baseline.schema.json` defines a contract version but lacks a test ensuring its constants align with truth in `crates/tokmd-analysis-types/src/lib.rs` and `docs/SCHEMA.md`. Adding tests prevents drift in these areas, fitting the Gatekeeper persona.

## Option A (recommended)
Add tests to `xtask/tests/docs_w43.rs` and `xtask/tests/docs_schema_w72.rs` to enforce that `BASELINE_VERSION` from `crates/tokmd-analysis-types/src/lib.rs` matches `BASELINE_VERSION` in `docs/SCHEMA.md`.
- What it is: A test-only change to tighten invariants.
- Why it fits: Matches Gatekeeper target 1 (schema/version drift) and 2 (snapshot/golden drift or weak coverage).
- Trade-offs: Minor test time increase; improves governance.

## Option B
Also try to validate `docs/baseline.schema.json` version.
- What it is: Extract the version from `baseline.schema.json` and verify it matches.
- When to choose: If we want to guarantee the schema's internal version matches the source of truth.
- Trade-offs: Slightly more complex test logic parsing JSON schema `$id` or `baseline_version` const.

## Decision
Choosing **Option A**. The tests have already been successfully prototyped and added, covering the basic alignment between source code and SCHEMA.md. Option B is also valuable and we can easily add it to `schema_json_receipt_versions_match_source` or a separate check. We will implement Option B as well (extending `schema_json_receipt_versions_match_source` for the baseline). Wait, `baseline.schema.json` is a separate file from `schema.json`. So we'll just add tests for `docs/SCHEMA.md` alignment. Option A is sufficient to close the gap.
