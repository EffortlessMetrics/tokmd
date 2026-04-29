# Decision

## Option A (recommended)
Add public sorting functions (`sort_lang_rows`, `sort_module_rows`, `sort_file_rows`) to `tokmd-model` and expose the inline sorting logic so that the determinism tests can directly use the actual library logic instead of defining identical closures in tests.
- **Why it fits**: The shard memory directly mentions: "In `tokmd`, sorting logic for data rows (like `LangRow`, `FileRow`) is implemented inline within aggregation functions in `tokmd-model` and `tokmd-format`. It is not exposed as standalone public functions (e.g., there are no `sort_lang_rows` functions). To test sorting determinism, tests must either invoke the parent aggregation functions or extract the inline logic into public functions, rather than redefining the sorting closures in the tests."
- **Trade-offs**:
  - *Structure*: Improves DRY and makes the public API more testable without redundant internal closures in the test file.
  - *Velocity*: Fast to implement.
  - *Governance*: Better lock-in for invariants.

## Option B
Test determinism only through the parent aggregation functions (`create_lang_report_from_rows`, etc.).
- **When to choose**: If exposing sorting functions bloats the public API undesirably.
- **Trade-offs**: More cumbersome to write focused property tests.

## Decision
**Option A**. It aligns perfectly with the explicit `Invariant` persona memory for this specific shard regarding determinism testing for `tokmd-model` sorting logic. We will extract `sort_lang_rows`, `sort_module_rows`, and `sort_file_rows` into public functions in `tokmd-model/src/lib.rs` and update `determinism_w66.rs` to use them.
