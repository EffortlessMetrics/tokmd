## 💡 Summary
I have added a property-based test `cocomo_effort_monotonically_increases_with_kloc` and `distribution_gini_is_zero_for_uniform_sizes` and `derive_report_is_deterministic` and `polyglot_entropy_is_zero_for_single_language` in `tokmd-analysis` to prove that COCOMO values scale monotonically, entropy values are precise, and reporting is deterministic. This tightens coverage around key invariants that are mentioned implicitly in the docs but lacking explicit proptests.

## 🎯 Why
The `derived` metrics include `cocomo`, `polyglot`, `distribution`, and other model properties. However, there were some gaps in the existing property tests:
1. `distribution.gini` was not explicitly verified to be `0.0` for identical line counts.
2. `cocomo.effort_pm` was not explicitly checked to be monotonically increasing for incremental LOC (Lines of Code) additions in property tests.
3. `polyglot.entropy` was missing a precise `0.0` bound check for a single language.
4. The determinism property `derive_report_is_deterministic` was missing across the whole `derived` module in `properties.rs`.

Tightening these bounds protects against silent regressions when metric constants or core parsing layers change.

## 🔎 Evidence
File path: `crates/tokmd-analysis/src/derived/tests/properties.rs`
Test execution receipt:
```text
test derived::tests::properties::cocomo_effort_monotonically_increases_with_kloc ... ok
test derived::tests::properties::distribution_gini_is_zero_for_uniform_sizes ... ok
test derived::tests::properties::derive_report_is_deterministic ... ok
test derived::tests::properties::polyglot_entropy_is_zero_for_single_language ... ok
test result: ok. 100 passed; 0 failed; 0 ignored; 0 measured; 1432 filtered out; finished in 6.36s
```

## 🧭 Options considered
### Option A (recommended)
- what it is: Add explicit proptests for gini bounds, polyglot entropy, derive determinism, and COCOMO monotonicity.
- why it fits this repo and shard: It operates purely in `tokmd-analysis` tests to lock in properties related to derived analysis without touching logic. Matches the `property` gate profile.
- trade-offs: Structure (keeps logic untouched) / Velocity (quick test additions) / Governance (reinforces determinism invariants).

### Option B
- what it is: Add a new integration test crate for testing module properties.
- when to choose it instead: If the properties required heavy test fixture data.
- trade-offs: Slows down execution times, introduces external dependencies.

## ✅ Decision
Option A. It adds the missing invariants while retaining existing execution boundaries and does not modify deterministic calculation paths.

## 🧱 Changes made (SRP)
- `crates/tokmd-analysis/src/derived/tests/properties.rs`
  - Added `cocomo_effort_monotonically_increases_with_kloc`
  - Added `distribution_gini_is_zero_for_uniform_sizes`
  - Added `polyglot_entropy_is_zero_for_single_language`
  - Added `derive_report_is_deterministic`

## 🧪 Verification receipts
```text
cargo test -p tokmd-analysis properties -- --nocapture
test derived::tests::properties::cocomo_effort_monotonically_increases_with_kloc ... ok
test derived::tests::properties::derive_report_is_deterministic ... ok
test derived::tests::properties::distribution_gini_is_zero_for_uniform_sizes ... ok
test derived::tests::properties::polyglot_entropy_is_zero_for_single_language ... ok
test result: ok. 100 passed; 0 failed; 0 ignored; 0 measured; 1432 filtered out
```

## 🧭 Telemetry
- Change shape: Test additions
- Blast radius: None (tests only)
- Risk class: Low - pure testing code
- Rollback: Revert changes in `crates/tokmd-analysis/src/derived/tests/properties.rs`
- Gates run: `cargo test -p tokmd-analysis`, `cargo clippy`, `cargo fmt`, `cargo xtask docs --check`

## 🗂️ .jules artifacts
- `.jules/runs/invariant_model_analysis/envelope.json`
- `.jules/runs/invariant_model_analysis/decision.md`
- `.jules/runs/invariant_model_analysis/receipts.jsonl`
- `.jules/runs/invariant_model_analysis/result.json`
- `.jules/runs/invariant_model_analysis/pr_body.md`

## 🔜 Follow-ups
None.
