# `tokmd-python` fails workspace clippy checks

## Description
The `tokmd-python` extension module causes the `cargo xtask gate --check` run to fail due to unused doc comments, unnecessary `match` statements where `if let` would be more idiomatic, and constant assertions (`assert!(true, ...)`). Since this package is part of the workspace, its clippy errors cause workspace-wide quality checks to fail.

## Context
When running `cargo xtask gate --check` or `cargo clippy --workspace --tests -- -D warnings`, the build breaks specifically inside `crates/tokmd-python/tests/property_tests.rs` and `crates/tokmd-python/src/lib.rs`.

## Suggested Fix
Apply the fixes suggested by clippy:
- Remove doc comments on property test macros in `crates/tokmd-python/tests/property_tests.rs`.
- Replace single-arm `match` expressions with `let _ = ...;` or `if let` in `crates/tokmd-python/src/lib.rs`.
- Remove assertions like `assert!(true, ...)` that evaluate to constants.
