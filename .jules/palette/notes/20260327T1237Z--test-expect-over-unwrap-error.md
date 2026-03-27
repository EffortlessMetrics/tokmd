# Test `expect` over `unwrap_err`

## Context
Tests often need to assert that an error is returned using `.unwrap_err()`. When this fails, the error message can be cryptic, especially if the expected error fails with a generic panic message. Similar issues occur when using `.unwrap()` without an explanation.

## Pattern
Replace `.unwrap_err()` with `.expect_err("descriptive message")` to provide a clear explanation of what the test expects and why it failed. The same logic applies to replacing `.unwrap()` with `.expect("should succeed")`.

## Evidence Pointers
- `crates/tokmd/tests/context_handoff_deep.rs`: Replaced many instances of `.unwrap()` with `.expect("should succeed")`.
- `crates/tokmd/src/context_pack.rs`: Replaced `.unwrap_err()` with `.expect_err("should return an error for invalid alpha")`.

## Prevention Guidance
When adding tests, try to use `expect()` and `expect_err()` to provide clear messaging rather than generic panic locations which takes time to trace and understand.
