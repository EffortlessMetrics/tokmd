# Prefer `.expect()` over `.unwrap()` in tests

**Context:** The  repository values DX. Panic messages from generic `.unwrap()` calls provide no context during test failures.
**Pattern:** We replaced `.unwrap()` with `.expect("specific message about the expected invariant")`.
**Evidence:** `crates/tokmd/tests/json_output.rs`, `crates/tokmd/tests/regression_suite_w52.rs`.
