# Test Determinism and Error Handling Pattern

**Date:** 2026-02-27T13:40:00Z
**Context:** During a security sweep, discovered prevalent use of `unwrap()` in tests for `tokmd-scan` and `tokmd-core`.
**Pattern:** Tests should avoid `unwrap()` on `Result` types where possible, instead returning `Result<(), Box<dyn std::error::Error>>` (or `anyhow::Result`). This allows tests to fail gracefully with descriptive error messages rather than panicking.
**Evidence:** `crates/tokmd-scan/src/lib.rs` and `crates/tokmd-core/src/ffi.rs` were refactored to use `?` and `expect("context")`.
**Guidance:**
1. Prefer `fn test_something() -> Result<()>` signatures for tests involving fallible operations.
2. Use `?` operator to propagate errors.
3. Use `expect("context")` for `Option` unwrapping or when `?` is not applicable, providing context for the failure.
**Links:**
- [Rust by Example: Question Mark](https://doc.rust-lang.org/rust-by-example/std/result/question_mark.html)
