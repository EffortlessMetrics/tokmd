## 💡 Summary
Added `#![cfg(feature = "analysis")]` to integration test files that explicitly test the `analysis` feature. This fixes a compilation and test failure when running `cargo test --no-default-features` on `tokmd`.

## 🎯 Why
When compiling or testing `tokmd` with `--no-default-features` (which excludes the `analysis` feature), the test framework still attempted to compile and run integration tests located in `tests/analyze_integration.rs`, `tests/badge_integration.rs`, `tests/baseline_integration.rs`, etc. These files run commands like `tokmd analyze` or `tokmd badge`, which fail with `Error: analysis feature is not enabled` under `--no-default-features`. By applying the standard Rust `cfg` attribute to these test modules, the tests are naturally skipped when the required features are not active, enabling a green test suite across the feature matrix.

## 🔎 Evidence
- **File path(s):** `crates/tokmd/tests/analyze_integration.rs`, `crates/tokmd/tests/badge_integration.rs`, `crates/tokmd/tests/baseline_integration.rs`, etc.
- **Observed behavior:** `cargo test -p tokmd --no-default-features` failed because tests in `analyze_integration.rs` panicked due to `Error: analysis feature is not enabled`.
- **Receipt:** Running `cargo test -p tokmd --no-default-features` now succeeds completely.

## 🧭 Options considered
### Option A (recommended)
- Add `#![cfg(feature = "analysis")]` to the top of the test files that exclusively test analysis features.
- **Why it fits:** It's the idiomatic Rust way to conditionally compile integration test files based on active features.
- **Trade-offs:**
  - **Structure:** Perfect alignment with Cargo features.
  - **Velocity:** Extremely fast and simple.
  - **Governance:** Clears up false positive failures in CI for feature matrix checks.

### Option B
- Modify the test cases to dynamically check for the presence of the feature.
- **When to choose it instead:** Only if the file tests a mix of feature-gated and non-feature-gated code.
- **Trade-offs:** High noise, test cases would be more complex and fragile.

## ✅ Decision
Chose Option A as it is the standard and most robust solution for conditionally compiled integration tests in Rust.

## 🧱 Changes made (SRP)
- Added `#![cfg(feature = "analysis")]` to `crates/tokmd/tests/analyze_integration.rs`
- Added `#![cfg(feature = "analysis")]` to `crates/tokmd/tests/badge_integration.rs`
- Added `#![cfg(feature = "analysis")]` to `crates/tokmd/tests/baseline_integration.rs`
- Added `#![cfg(feature = "analysis")]` to `crates/tokmd/tests/baseline_w71.rs`
- Added `#![cfg(feature = "analysis")]` to `crates/tokmd/tests/bdd_analyze_scenarios_w50.rs`
- Added `#![cfg(feature = "analysis")]` to `crates/tokmd/tests/cli_badge_e2e.rs`
- Added `#![cfg(feature = "analysis")]` to `crates/tokmd/tests/cockpit_cli_w71.rs`
- Added `#![cfg(feature = "analysis")]` to `crates/tokmd/tests/cockpit_cli_w75.rs`
- Added `#![cfg(feature = "analysis")]` to `crates/tokmd/tests/cockpit_integration.rs`
- Added `#![cfg(feature = "analysis")]` to `crates/tokmd/tests/gate_integration.rs`
- Added `#![cfg(feature = "analysis")]` to `crates/tokmd/tests/run_diff.rs`

## 🧪 Verification receipts
```text
cargo test -p tokmd --no-default-features
...
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.95s
```

## 🧭 Telemetry
- **Change shape:** Rust module-level `cfg` attributes added to integration tests.
- **Blast radius:** Minimal. Only affects test compilation.
- **Risk class:** Very low. Tests are skipped correctly under no-default-features and continue to run under normal feature sets.
- **Rollback:** Safe to revert, though it would reintroduce the test failure.
- **Gates run:** `cargo test -p tokmd --no-default-features` and `cargo test -p tokmd`.

## 🗂️ .jules artifacts
- `.jules/runs/compat_interfaces_matrix/envelope.json`
- `.jules/runs/compat_interfaces_matrix/decision.md`
- `.jules/runs/compat_interfaces_matrix/receipts.jsonl`
- `.jules/runs/compat_interfaces_matrix/result.json`
- `.jules/runs/compat_interfaces_matrix/pr_body.md`

## 🔜 Follow-ups
None.
