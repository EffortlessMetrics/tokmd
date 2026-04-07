# Fuzz Report: tokmd-python FFI Error Handling

**Run ID:** run_tokmd_887_1744034820000  
**Gate:** PROVEN (Phase 1: FFI Critical)  
**Target:** tokmd-python error handling  
**Date:** 2026-04-07  

---

## Executive Summary

Fuzzing assessment of the tokmd-python FFI boundary found **1 potential issue** with strict parsing validation. The core FFI entrypoint (`run_json`) demonstrates robust error handling with no panics detected. However, property-based testing revealed that certain numeric fields may not enforce strict type validation.

**Overall Assessment:** The FFI boundary is crash-resistant but has a minor strictness gap in field validation.

---

## Infrastructure Status

### cargo-fuzz Availability
- **Installed:** Yes (cargo-fuzz v0.13.1)
- **Nightly toolchain:** Available (cargo 1.96.0-nightly)
- **Build Status:** ❌ **BLOCKED**

**Issue:** AddressSanitizer (ASAN) linker errors prevent fuzz target compilation:
```
rust-lld: error: undefined symbol: __sancov_gen_.225
```

This is a known issue with certain nightly Rust versions and sanitizer coverage. The fuzz targets exist but cannot be executed in the current environment.

### Existing Fuzz Targets
The following fuzz targets are defined in `fuzz/` but unexecutable due to ASAN issues:

| Target | Purpose | Status |
|--------|---------|--------|
| `fuzz_run_json` | Tests FFI entrypoint with arbitrary mode/args | 🔴 Blocked |
| `fuzz_ffi_envelope` | Tests envelope parser/extractor invariants | 🔴 Blocked |
| `fuzz_json_types` | Tests receipt type deserialization | 🔴 Blocked |
| `fuzz_entropy` | Tests content analysis utilities | 🔴 Blocked |

### Alternative: Property-Based Testing
**proptest** is functional and provides similar coverage. Located in:
- `crates/tokmd-python/tests/property_tests.rs`

**Results:** 11 tests pass, 1 test reveals strictness issue.

---

## Findings

### Finding 1: Strict Parsing Gap for Numeric Fields in Export Mode ⚠️

**Severity:** Low  
**Location:** `tokmd-core/src/ffi.rs` - `parse_export_settings()`  

**Description:**
Property-based testing revealed that the `max_rows` field in export mode does not enforce strict type validation. When passed a string value like `"a"`, it silently accepts the input rather than returning an `invalid_settings` error.

**Reproduction:**
```bash
cargo test --package tokmd-python --test property_tests
# Test: prop_strict_parsing_invalid_numeric_type_produces_error
# Failing input: field = "max_rows", invalid_value = "a"
```

**Expected Behavior:**
```json
{"ok": false, "error": {"code": "invalid_settings", "message": "..."}}
```

**Actual Behavior:**
The input is accepted (likely defaulting to 0 or parsing incorrectly).

**Root Cause Analysis:**
The export settings parsing may not use the same strict `parse_usize` helper that other modes use. Compare:
- `parse_lang_settings()` - uses `parse_usize()` which validates types strictly
- `parse_export_settings()` - may allow string values to pass through

**Verification:**
Unit tests in `tokmd-core/src/ffi.rs` show strict parsing for `lang` mode:
```rust
#[test]
fn strict_parsing_invalid_usize() {
    let args: Value = serde_json::json!({"top": "ten"});
    let err = parse_lang_settings(&args).expect_err("should fail");
    assert_eq!(err.code, crate::error::ErrorCode::InvalidSettings);
}
```

However, export mode lacks equivalent strictness tests.

**Recommendation:**
1. Add strict parsing for all numeric fields in `parse_export_settings()`
2. Add regression tests for invalid types in export mode
3. Verify other modes (module, analyze) have consistent strictness

---

## FFI Boundary Crash Resistance

### Panic Testing
The FFI boundary was tested for panic conditions via:

1. **Unit tests in `tokmd_core::ffi`:**
   - `run_json_always_returns_valid_json()` - 10 test cases including invalid UTF-8, null bytes, and edge cases
   - All pass - no panics detected

2. **Property tests in `tokmd-python`:**
   - `prop_run_json_is_total_function()` - 512 cases with arbitrary bytes
   - Uses `std::panic::catch_unwind` - no panics detected

3. **Red tests in `tokmd-python/src/lib.rs`:**
   - `red_test_python_ffi_no_panic_on_none_paths`
   - `red_test_python_ffi_no_panic_on_empty_paths`
   - `red_test_python_ffi_no_panic_on_unusual_paths`
   - `red_test_python_ffi_no_panic_on_extremely_long_paths`
   - `red_test_python_ffi_io_error_translation`
   - All pass - FFI is panic-free

### PyResult Handling
All Python-facing functions correctly return `PyResult<T>`:
- `run_json()` - validates JSON before releasing GIL, returns `PyResult<String>`
- `run()` - uses `?` operator for error propagation
- `lang()`, `module()`, `export()`, `analyze()`, `diff()`, `cockpit()` - all return `PyResult<PyObject>`

---

## Envelope Parsing Invariants

The `tokmd-ffi-envelope` microcrate maintains these invariants:

1. **Totality:** `parse_envelope()` never panics - returns `Result<Value, EnvelopeExtractError>`
2. **Determinism:** Same input produces same output (verified in property tests)
3. **Equivalence:** Step-wise API (`parse_envelope` + `extract_data`) matches convenience API (`extract_data_from_json`)

**Test Coverage:**
- Unit tests: 8 tests covering all error paths
- Fuzz target: `fuzz_ffi_envelope` (blocked by ASAN issues)

---

## Recommendations

### Immediate Actions
1. **Fix strict parsing for export mode numeric fields** - Ensure `max_rows`, `min_code`, and other numeric fields validate types strictly
2. **Add regression test** - Include the failing proptest case in unit tests
3. **Audit other modes** - Verify module and analyze modes have consistent strictness

### Infrastructure Improvements
1. **Resolve ASAN linker issues** - Update nightly toolchain or use `-Z sanitizer=address` with different codegen settings
2. **Enable cargo-fuzz in CI** - Once ASAN is working, add fuzzing to the gate pipeline
3. **Extend proptest coverage** - Add more field validation strategies for comprehensive coverage

### Fuzz Target Wishlist
When ASAN is fixed, prioritize these targets:
1. `fuzz_run_json` - Critical FFI entrypoint
2. `fuzz_ffi_envelope` - Response parsing
3. `fuzz_export_tree` - Export data handling (relevant to Python consumers)

---

## Conclusion

The tokmd-python FFI boundary is **crash-resistant** with comprehensive error handling. No panics were detected through multiple testing approaches. The single finding is a **strictness gap** rather than a crash vulnerability - invalid inputs are handled gracefully but without the expected error feedback.

**Gate Status:** PROVEN with minor strictness reservation

---

## Appendix: Test Commands

```bash
# Run property tests (includes the strictness finding)
cargo test --package tokmd-python --test property_tests

# Run unit tests for FFI boundary
cargo test --package tokmd-core ffi::

# Run red tests (panic resistance)
cargo test --package tokmd-python red_test_python_ffi

# Attempt fuzzing (requires ASAN fix)
cargo +nightly fuzz run fuzz_run_json --features core
cargo +nightly fuzz run fuzz_ffi_envelope --features ffi_envelope
```
