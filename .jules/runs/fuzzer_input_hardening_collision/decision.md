# Decision

## Option A (Add properties test in tokmd crate)
Add the deterministic proptest coverage for `scan_args` to `crates/tokmd/tests/properties.rs`.

## Option B (Abort superseded work)
Acknowledge PR feedback indicating `scan_args` is already fully covered in `crates/tokmd-format/src/scan_args/mod.rs` on `origin/main`. Revert changes, record a friction item mapping this collision, and exit gracefully with a learning PR.

## Decision: Option B
The test duplication and incorrect boundary assignment (`schema_contracts`) were caught in review. Per instructions, gracefully abort superseded work and submit a learning PR.
