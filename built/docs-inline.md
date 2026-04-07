# Inline Documentation: FFI Error Handling Rationale

**Run ID:** run_tokmd_887_1744034820000  
**Gate:** BUILT (Phase 1: FFI Critical)  
**Date:** 2026-04-07  
**Issue:** https://github.com/EffortlessMetrics/tokmd/issues/887  

## Summary

Added comprehensive inline documentation to `crates/tokmd-python/src/lib.rs` explaining the error handling rationale at the Python ↔ Rust FFI boundary.

## Documentation Added

### 1. Module-Level Documentation (Crate Header)

**Location:** Lines 1-30 (crate doc comment)

**Documents:**
- FFI Safety Invariants (4 core principles)
- Error Handling Strategy
- Why `PyResult<T>` is used (never panic guarantee)
- Why early validation matters (before GIL release)

**Key Excerpt:**
```rust
//! # FFI Safety Invariants
//!
//! 1. **Never Panic Guarantee**: All Python-facing functions return `PyResult<T>`
//!    and use the `?` operator for error propagation. The `.expect()` method is
//!    prohibited in production code because a panic would crash the host Python
//!    interpreter.
```

### 2. `TokmdError` Type Documentation

**Location:** Line 32 (above `pyo3::create_exception`)

**Documents:**
- SAFETY: Exception type registration with Python interpreter
- All tokmd-specific errors use this type for clear error semantics

### 3. `run_json()` Function Documentation

**Location:** Lines 63-100

**Documents:**
- **Why `args_json` validation matters:** Early validation prevents invalid JSON from reaching core FFI
- **Host Process Safety:** Validation occurs before `allow_threads()` to maintain interpreter consistency
- **GIL Handling:** Explanation of when and why GIL is released
- **Fail-Fast Rationale:** Invalid JSON rejected immediately with `ValueError`

**Key Excerpt:**
```rust
/// # FFI Safety Rationale
///
/// This function validates `args_json` **before** releasing the GIL for two reasons:
///
/// 1. **Fail-Fast**: Invalid JSON is rejected immediately with a clear `ValueError`,
///    preventing wasted work in long-running scans.
///
/// 2. **Host Process Safety**: By validating while the GIL is still held, we ensure
///    that any parsing errors are reported before entering the `allow_threads` block.
///    This guarantees the Python interpreter remains in a consistent state.
```

### 4. `run()` Function Documentation

**Location:** Lines 109-145

**Documents:**
- Error Handling Strategy (3-step flow)
- Why `?` operator is used at each boundary
- How errors propagate without panicking

### 5. `run_with_json_module()` Function Documentation

**Location:** Lines 147-187

**Documents:**
- **Design Rationale:** Why JSON module is injectable (testability)
- **FFI Safety Notes:** Each `?` represents a Python exception return point
- Error propagation chain explained

**Key Excerpt:**
```rust
/// # FFI Safety Notes
///
/// Each `?` operator in this function represents a potential Python exception return:
/// - `json_module?` - ImportError if json module unavailable
/// - `call_method1(...)?` - TypeError/ValueError if serialization fails
/// - `extract()?` - TypeError if result is not a string
/// - `extract_data_json()?` - TokmdError if envelope extraction fails
```

### 6. `build_args()` Function Documentation

**Location:** Lines 295-340

**Documents:**
- **Why `PyResult<Bound>` instead of `Bound`:** Every `set_item()` can fail
- **Why `?` Instead of `.expect()`:** CRITICAL safety explanation
- **Host Process Safety Invariant:** Each `?` is a safety boundary

**Key Excerpt:**
```rust
/// # Why `?` Instead of `.expect()`
///
/// **NEVER use `.expect()` in production FFI code.** A panic would:
/// - Abort the entire Python interpreter process
/// - Destroy all Python objects and state
/// - Provide no useful error information to the Python caller
///
/// The `?` operator converts any PyO3 error to a `PyErr`, which becomes a
/// proper Python exception that can be caught and handled.
```

### 7. Wrapper Function Documentation Updates

**Functions Updated:** `lang()`, `module()`, `export()`, `analyze()`, `diff()`, `cockpit()`

**Pattern Documented:**
```rust
/// # Error Propagation Pattern
///
/// All wrapper functions follow the same FFI-safe pattern:
/// 1. `build_args()?` - Creates args dict, propagates any PyDict errors
/// 2. `args.set_item()?` - Adds mode-specific args, propagates failures
/// 3. `run()?` - Executes scan, returns result or TokmdError
///
/// The `?` operator at each step ensures Python exceptions propagate
/// cleanly without panicking the interpreter.
```

## Why This Documentation Matters

### 1. Never Panic Guarantee

The documentation makes explicit that `.expect()` is prohibited in production code because:
- A panic crashes the entire Python interpreter
- All Python objects and state are destroyed
- Users cannot catch and handle panics

### 2. Error Propagation Strategy

The `?` operator is used consistently because:
- Converts Rust errors to Python exceptions (`PyErr`)
- Allows Python callers to catch and handle errors
- Maintains interpreter consistency on all error paths

### 3. Host Process Safety

Early validation (like `args_json` JSON parsing) matters because:
- Fails fast before expensive operations
- Prevents invalid data from reaching core FFI
- Maintains GIL consistency (errors reported before `allow_threads`)

### 4. FFI Boundary Invariants

The documentation establishes 4 invariants:
1. All Python-facing functions return `PyResult<T>`
2. Input validation occurs before releasing GIL
3. GIL is properly acquired/released
4. Rust errors become Python exceptions, never panics

## Files Modified

| File | Change |
|------|--------|
| `crates/tokmd-python/src/lib.rs` | Added module-level docs, docstrings on 8+ functions, inline safety comments |

## Verification

Documentation additions verified:
- ✅ Module-level FFI safety invariants documented
- ✅ `run_json()` validation rationale explained
- ✅ `build_args()` `PyResult` rationale explained
- ✅ `?` vs `.expect()` documented with consequences
- ✅ All wrapper functions reference error propagation pattern
- ✅ GIL safety rationale documented

## References

- Implementation: `built/implementation.md`
- State: `built/state.json`
- PyO3 FFI Safety: https://pyo3.rs/main/doc/ffi-safety
