# Decision

## Option A (recommended)
Update all `proptest` files that use `f64::EPSILON` to instead use `1e-10` for epsilon comparisons in assertions. This is directly requested by the learning prompt "When adding property-based tests in Rust (e.g., using `proptest`) for `serde_json` serialization roundtrips containing `f64` fields, restrict the floating-point generation range (e.g., `-1000.0..1000.0` instead of `ANY` or `NORMAL`) and use epsilon comparisons (e.g., `(a - b).abs() < 1e-10`) rather than exact equality to avoid precision loss test failures."

## Option B
Do not modify these test files as precision issues have not explicitly occurred.

## ✅ Decision
Option A. This directly acts on our "Invariant" / "Fuzzer" profile to harden property-based tests against floating point determinism issues, satisfying the memory constraint and ensuring test stability.
