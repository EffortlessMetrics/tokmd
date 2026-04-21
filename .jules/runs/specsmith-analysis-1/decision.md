# Decision

## Option A (recommended)
Update `is_test_path` in `tokmd-analysis-util/src/lib.rs` to detect standard standalone test files such as `test.*`, `tests.*`, `spec.*`, and `specs.*` (e.g., `test.rs`, `tests.py`). Currently, these standalone files are completely missed by the heuristic which only handles test directory components and prefix/suffix underscores.

We will add these cases to `is_test_path` and update BDD tests and properties to verify these scenarios, increasing test coverage.

## Option B
Leave the logic as is and rely on users to name their standalone test files `test_foo.rs` or put them in `tests/` directories. This is less ideal as many ecosystems use `test.js` or `tests.py` directly.

## Decision
Option A. It's an important edge-case regression/gap not locked in by tests, perfectly aligning with Specsmith's mission to improve scenario coverage and regression coverage.
