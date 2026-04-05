# Boundary Crate Unwrap Burndown

**Context**: The `tokmd-ffi-envelope` acts as an FFI boundary parser.
**Pattern**: Generic `.unwrap()` or `.unwrap_err()` calls were used in its tests and documentation examples.
**Guidance**: Replaced with `.expect("message")` and `.expect_err("message")` to explicitly document the tested invariants (e.g., "valid JSON string should parse") and prevent confusing, context-free panics during test failures.
