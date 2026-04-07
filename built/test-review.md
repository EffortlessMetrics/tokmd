# Test Review: FFI Module (Gate 4: BUILT)

**Run ID:** run_tokmd_887_1744034820000  
**Component:** `crates/tokmd-core/src/ffi.rs`  
**Test Count:** 35 total (31 pass in default config, 4 feature-gated)  
**Review Date:** 2026-04-07  
**Reviewer:** Hearth Conveyor (Test Reviewer Agent)

---

## Executive Summary

**Verdict: APPROVED WITH MODIFICATIONS**

The FFI test suite is unexpectedly comprehensive. Tests are well-structured, verify real behavior (not just "doesn't panic"), and define clear contracts for error codes and messages. However, several gaps exist around edge cases and the "red test" contract verification needs clarification.

---

## Coverage Assessment

### ✅ Strong Coverage Areas

| Area | Test Count | Quality |
|------|-----------|---------|
| Response envelope structure | 3 | High - validates `ok` field always present |
| Error code specificity | 8 | High - each test asserts exact `error.code` |
| Settings parsing strictness | 10 | High - verifies type errors, not just presence |
| Feature gating | 4 | Medium - tests both with/without analysis feature |
| Default value handling | 4 | Medium - null → default behavior verified |
| Nested object parsing | 2 | Medium - `scan.*` and `lang.*` nesting works |

### ⚠️ Coverage Gaps

| Gap | Risk Level | Notes |
|-----|-----------|-------|
| **Invalid UTF-8 input** | HIGH | No test for non-UTF8 bytes in `args_json` |
| **Null pointer/empty string mode** | MEDIUM | Empty mode `""` tested but not null ptr scenario |
| **Very large JSON payloads** | MEDIUM | No test for 10MB+ inputs that could OOM |
| **Unicode edge cases** | LOW | No test for U+0000, combining chars in paths |
| **Concurrent/parallel calls** | LOW | No test for thread safety of `run_json` |
| **In-memory inputs validation** | MEDIUM | `parse_in_memory_inputs` has no dedicated tests |

### Test Categories Breakdown

```
35 total tests:
├── 4  - Basic API/version tests
├── 4  - Settings defaults tests
├── 8  - Strict parsing validation tests
├── 3  - Envelope/invariant tests (incl. totality)
├── 4  - Nested object & null handling
├── 1  - Array error precision
├── 2  - Diff field validation
├── 5  - Analysis feature-gated tests
├── 3  - Cockpit settings tests
├── 1  - Cockpit feature-gated test
```

---

## Test Quality Analysis

### ✅ What Tests Do Well

1. **Assert on error codes, not just failure:**
   ```rust
   assert_eq!(parsed["error"]["code"], "invalid_settings");  // Not just "err != null"
   ```

2. **Validate error messages contain field names:**
   ```rust
   assert!(err.message.contains("paths[1]"));  // Index-specific error location
   ```

3. **Totality invariant test is excellent:**
   - `run_json_always_returns_valid_json` tests 11 edge cases including:
     - Empty mode and args
     - Invalid JSON (`"not valid json"`)
     - Non-object JSON (`null`, `[]`, `123`)
     - Negative numbers
     - Null bytes in mode (`"\0"`)
     - Wrong array element types

4. **Strict parsing tests verify type coercion is rejected:**
   - `"hidden": "yes"` → error (not silent true)
   - `"top": "ten"` → error (not silent 0)

### ⚠️ Quality Issues

1. **Some tests only check string containment, not exact match:**
   ```rust
   assert!(err.message.contains("hidden"));  // Could pass with weak message
   ```

2. **Feature-gated tests don't run in default CI:**
   - `invalid_analyze_preset_returns_error` requires `analysis` feature
   - These tests may rot if CI doesn't build with `--all-features`

3. **No test for the actual FFI boundary:**
   - Tests use `run_json()` directly (Rust-to-Rust)
   - No C ABI tests for the actual `extern "C"` boundary if one exists

---

## Risks: Could Tests Pass with Broken Implementation?

### 🔴 High Risk

| Risk | Scenario | Mitigation |
|------|----------|------------|
| Silent data loss on invalid UTF-8 | If input has invalid UTF-8, `serde_json::from_str` may mangle it before parsing | Add test with `std::str::from_utf8_lossy` comparison |
| Envelope structure change | Tests check `parsed["ok"]` but not full envelope schema | Add schema validation test |

### 🟡 Medium Risk

| Risk | Scenario |
|------|----------|
| Feature gate drift | Tests with `#[cfg(not(feature = "analysis"))]` may pass when feature IS enabled, masking breakage |
| Default value assumptions | Tests assume `top: 0` default but don't verify this is documented behavior |

### 🟢 Low Risk

- Basic functionality is well-covered
- Error envelope contract is enforced
- Type safety prevents most FFI memory issues

---

## Recommendations

### Before APPROVED → VERIFIED transition:

1. **Add 3 missing edge case tests:**
   ```rust
   #[test]
   fn invalid_utf8_in_json_returns_error() { ... }
   
   #[test] 
   fn in_memory_inputs_requires_path_and_content() { ... }
   
   #[test]
   fn empty_mode_returns_unknown_mode_error() {  // "" already tested, verify error code
       let result = run_json("", r#"{"paths": ["."]}"#);
       // assert specific error code
   }
   ```

2. **Document feature-gated test strategy:**
   - Add CI job that runs tests with `--all-features`
   - Or explicitly document that analysis/cockpit tests are manual-only

3. **Add envelope schema contract test:**
   ```rust
   #[test]
   fn error_envelope_has_required_fields() {
       // Verify: ok, error.code, error.message always present
       // Verify: data only present when ok=true
   }
   ```

---

## Verdict

**Status: APPROVED WITH MODIFICATIONS**

The test suite is stronger than typical "red tests" - it actually verifies behavior and defines contracts. The 35 tests cover:
- ✅ Error code specificity
- ✅ Strict parsing rejection of invalid types  
- ✅ Envelope totality invariant
- ✅ Feature-gated behavior
- ⚠️ Missing: UTF-8 edge cases, in-memory input validation

**Gate 4 (BUILT) can proceed to next phase with the understanding that 3 additional edge case tests should be added before VERIFIED gate.**

---

## References

- Test file: `crates/tokmd-core/src/ffi.rs` (lines 635-1112)
- Error types: `crates/tokmd-core/src/error.rs`
- Command: `cargo test --package tokmd-core --lib ffi::tests`
- Issue: https://github.com/EffortlessMetrics/tokmd/issues/887
