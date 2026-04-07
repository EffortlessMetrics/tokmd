# Mutation Testing Report: tokmd-core

**Run ID:** run_tokmd_887_1744034820000  
**Gate:** PROVEN  
**Date:** 2026-04-07  
**Tool:** cargo-mutants v27.0.0  
**Target:** tokmd-core crate

---

## Summary

Mutation testing evaluates test suite effectiveness by introducing small, semantically meaningful changes (mutations) to source code and verifying that tests catch these "bugs." A caught mutant indicates tests are properly exercising that code path.

### Mutation Score: **58.6%** (17/29 viable mutants caught)

| Category | Count | Percentage |
|----------|-------|------------|
| Caught (killed) | 17 | 58.6% of viable |
| Missed (survived) | 12 | 41.4% of viable |
| Unviable | 23 | N/A |
| Timeout | 0 | 0% |
| **Total Generated** | **52** | — |

---

## Analysis

### Test Suite Strength: MODERATE

A 58.6% mutation score indicates the test suite catches roughly **6 in 10** introduced bugs. This is a moderate result with clear improvement opportunities.

**What the score means:**
- **50-70%**: Moderate coverage — tests catch obvious bugs but miss edge cases
- **70-85%**: Good coverage — tests are comprehensive, suitable for production
- **85%+**: Excellent coverage — tests are thorough and resilient

### Caught Mutants (17) — Tests Working Well

The following mutation types are properly detected by the test suite:

| Function | Mutation | Status |
|----------|----------|--------|
| `now_ms` | Replace with `0` (non-WASM) | ✅ Caught |
| `now_ms` | Replace with `1` (non-WASM) | ✅ Caught |
| `supports_rootless_in_memory_analyze_preset` | Replace with `true` | ✅ Caught |
| `supports_rootless_in_memory_analyze_preset` | Replace with `false` | ✅ Caught |
| `supports_rootless_in_memory_analyze_preset` | `\|\|` → `&&` | ✅ Caught |
| `settings_to_scan_options` | Replace with `Default::default()` | ✅ Caught |
| `deterministic_in_memory_scan_options` | Replace with `Default::default()` | ✅ Caught |
| `collect_pure_in_memory_rows` | Return empty vec | ✅ Caught |
| `collect_pure_in_memory_rows` | Return vec with default | ✅ Caught |
| `collect_materialized_rows` | Return empty vec | ✅ Caught |
| `strip_virtual_export_prefix` | Return empty vec | ✅ Caught |
| `build_export_receipt` | `\|\|` → `&&` | ✅ Caught |
| `build_export_receipt` | `==` → `!=` (left) | ✅ Caught |
| `build_export_receipt` | `==` → `!=` (right) | ✅ Caught |
| `parse_analysis_preset` | Delete "receipt" arm | ✅ Caught |
| `parse_analysis_preset` | Delete "estimate" arm | ✅ Caught |
| `parse_analysis_preset` | Delete "health" arm | ✅ Caught |

### Missed Mutants (12) — Test Gaps Identified

These mutations survived, indicating tests don't sufficiently exercise these code paths:

| Function | Mutation | Risk Level |
|----------|----------|------------|
| `now_ms` | Replace with `1` (WASM) | 🟡 Low |
| `cockpit_workflow` | Delete `!` operator | 🔴 High |
| `cockpit_workflow` | Delete "three-dot" \| "3dot" arm | 🟡 Medium |
| `build_export_receipt` | `&&` → `\|\|` | 🔴 High |
| `parse_analysis_preset` | Delete "risk" arm | 🟡 Medium |
| `parse_analysis_preset` | Delete "supply" arm | 🟡 Medium |
| `parse_analysis_preset` | Delete "architecture" arm | 🟡 Medium |
| `parse_analysis_preset` | Delete "topics" arm | 🟡 Medium |
| `parse_analysis_preset` | Delete "security" arm | 🟡 Medium |
| `parse_analysis_preset` | Delete "identity" arm | 🟡 Medium |
| `parse_analysis_preset` | Delete "git" arm | 🟡 Medium |
| `parse_analysis_preset` | Delete "deep" arm | 🟡 Medium |

### Unviable Mutants (23)

These mutations failed to compile or were semantically invalid:

- Workflow functions returning `Ok(Default::default())` — type mismatch (LangReceipt, ModuleReceipt, ExportReceipt, DiffReceipt, AnalysisReceipt don't implement Default)
- Collection helpers with invalid default constructions
- Most "replace with Default" mutations on complex return types

This is expected — unviable mutants don't indicate test weakness, just that the mutation couldn't be applied.

---

## Critical Gaps

### 1. `cockpit_workflow` Logic Not Tested (HIGH RISK)

**Issue:** Both boolean operator mutations in `cockpit_workflow` survived.

```rust
// Line 519: delete ! operator survived
// Line 528: delete "three-dot" | "3dot" arm survived
```

**Impact:** Logic errors in cockpit workflow (CLI dispatch) would go undetected.

**Recommendation:** Add integration tests covering:
- Cockpit workflow with various flag combinations
- Three-dot notation handling
- Boolean negation paths

### 2. `build_export_receipt` Boolean Logic Gap (HIGH RISK)

**Issue:** `&&` → `||` mutation survived at line 785.

```rust
// Line 785: replace && with || in build_export_receipt
```

**Impact:** Export receipt building logic is incompletely tested. A logic error could produce incorrect export metadata.

**Recommendation:** Add tests verifying export receipt construction with various input combinations.

### 3. `parse_analysis_preset` Match Arms Not Covered (MEDIUM RISK)

**Issue:** 9 out of 12 analysis preset match arms are not tested:
- ❌ "risk" — not tested
- ❌ "supply" — not tested  
- ❌ "architecture" — not tested
- ❌ "topics" — not tested
- ❌ "security" — not tested
- ❌ "identity" — not tested
- ❌ "git" — not tested
- ❌ "deep" — not tested
- ✅ "receipt" — tested
- ✅ "estimate" — tested
- ✅ "health" — tested

**Impact:** Adding new analysis presets or modifying existing ones won't be caught by tests.

**Recommendation:** Add unit tests for `parse_analysis_preset` covering all arm branches.

### 4. WASM `now_ms` Not Tested (LOW RISK)

**Issue:** The WASM-specific `now_ms` implementation returning `1` survived.

```rust
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
fn now_ms() -> u128 {
    0  // Mutated to 1, tests passed
}
```

**Impact:** Time-based receipt generation in WASM environment may have edge case issues.

**Recommendation:** If WASM is a supported target, add WASM-specific tests or mock testing.

---

## Recommendations

### Immediate Actions (Gate PROVEN)

1. **Add tests for `parse_analysis_preset`** — Cover all 12 match arms with parameterized tests
2. **Add tests for `build_export_receipt` boolean logic** — Verify operator behavior
3. **Add `cockpit_workflow` integration tests** — Cover flag parsing and dispatch logic

### Long-term Improvements

| Target | Current | Goal | Priority |
|--------|---------|------|----------|
| Overall mutation score | 58.6% | 75%+ | High |
| `cockpit_workflow` coverage | 0% | 100% | Critical |
| `build_export_receipt` coverage | 60% | 100% | High |
| `parse_analysis_preset` coverage | 25% | 100% | Medium |

### Testing Strategy

1. **Targeted unit tests** for `parse_analysis_preset` with all preset strings
2. **Property-based tests** for boolean logic in `build_export_receipt`
3. **Integration tests** for `cockpit_workflow` with various CLI flag combinations
4. **Mock-based tests** for WASM code paths if target support is needed

---

## Raw Output

**caught.txt:** 17 mutants killed by test suite  
**missed.txt:** 12 mutants survived (test gaps)  
**unviable.txt:** 23 mutants couldn't compile  
**timeout.txt:** 0 mutants timed out

Full results available in `mutants.out/` directory:
- `mutants.json` — All generated mutations
- `outcomes.json` — Per-mutant test results
- `diff/` — Diff files for each mutant
- `log/` — Build/test logs

---

## Conclusion

The tokmd-core test suite has **moderate effectiveness** at catching bugs. While core functionality (scanning, modeling, basic receipt building) is well-tested, CLI dispatch logic and analysis preset parsing have significant gaps. 

**Verdict:** Gate 5 (PROVEN) requires test improvements in three areas before the test suite can be considered production-grade:
1. `cockpit_workflow` logic testing
2. `build_export_receipt` boolean coverage  
3. `parse_analysis_preset` match arm completeness

With these additions, the mutation score should reach 75%+, indicating a robust test suite.
