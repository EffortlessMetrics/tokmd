## Summary

Add 116 targeted mutation-killing tests across four critical crates to improve mutation testing scores and protect against subtle behavioral regressions.

### Crates Covered

| Crate | Tests | Key Mutation Targets |
|-------|-------|---------------------|
| **tokmd-redact** | 20 | Hash truncation boundary (16 vs 15/17), separator normalization, extension preservation, privacy guarantees |
| **tokmd-gate** | 48 | Comparison operator boundaries (> vs >=, < vs <=), negate logic, from_results counting, fail_fast conjunctions, ratchet percentage arithmetic, missing value handling |
| **tokmd-model** | 20 | avg() division-by-zero guard, rounding arithmetic, normalize_path separator handling, module_key depth logic |
| **tokmd-types** | 28 | Schema version constant pinning (all 5 constants), TokenEstimationMeta ceil-vs-floor arithmetic, TokenAudit saturating_sub, default trait impls, serde roundtrips |

### Mutation Patterns Targeted

- **Boundary conditions**: Exact values where `>` vs `>=` (or `<` vs `<=`) produce different results
- **Arithmetic mutations**: `+` to `-`, `*` to `/`, `ceil` to `floor`
- **Boolean logic**: `&&` to `||`, `!` removal, `true`/`false` flips
- **Guard removal**: `if x == 0 { return 0 }` deletion
- **Constant mutations**: Schema version ±1 changes
- **Method removal**: `replace()`, `strip_prefix()`, `saturating_sub()` deletions

### Testing

All 116 tests pass. No existing tests affected. Clippy clean.
