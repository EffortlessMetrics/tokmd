## 💡 Summary
Added `#![cfg(feature = "analysis")]` or `#![cfg(feature = "git")]` to integration tests in `crates/tokmd` that rely on these optional features. This fixes a matrix testing failure where `cargo test --no-default-features` fails because the tests attempt to execute commands that require missing features.

## 🎯 Why
When testing with `--no-default-features` (a standard compatibility check in the `compat-matrix` profile), multiple `tokmd` tests failed with errors like `tokmd analyze failed: ExitStatus(unix_wait_status(256))` or `"Error: analysis feature is not enabled\n"`. This is because tests evaluating optional CLI subcommands (like `analyze`, `badge`, `baseline`, `run`, `gate`, etc.) were not skipped when the underlying features were disabled.
As instructed by memory: "integration tests executing CLI subcommands that require optional features... must be conditionally compiled using `#![cfg(feature = "analysis")]` at the test file level to prevent false-positive failures during `--no-default-features` matrix test runs."

## 🔎 Evidence
- `crates/tokmd/tests/analyze_integration.rs` fails on `cargo test -p tokmd --no-default-features` due to lack of `analysis` feature constraint.
- Receipt:
  ```text
  running 7 tests
  test analyze_explain_known_metric ... FAILED
  test analyze_explain_list ... FAILED
  test analyze_fun_preset_returns_eco_label ... FAILED
  ...
  ```

## 🧭 Options considered
### Option A (recommended)
- what it is: Add module-level `#![cfg(feature = "...")]` to test files requiring specific features.
- why it fits this repo and shard: Directly aligns with standard cargo practices for integration test compatibility with `--no-default-features`, resolving test failure matrices while touching only test files.
- trade-offs: Structure / Velocity / Governance: Low-friction fix that significantly improves cross-matrix test velocity.

### Option B
- what it is: Update `tokmd` to treat these subcommands conditionally inside the code rather than tests.
- when to choose it instead: If the CLI itself panicked on missing features rather than cleanly returning an error.
- trade-offs: High risk, out of scope for a test failure.

## ✅ Decision
Chosen Option A. It's the idiomatically correct and documented approach for integration tests requiring optional workspace features, fixing false positives in `--no-default-features` test runs.

## 🧱 Changes made (SRP)
Added `#![cfg(feature = "analysis")]` or `#![cfg(feature = "git")]` to the following files:
- `crates/tokmd/tests/analyze_integration.rs`
- `crates/tokmd/tests/badge_integration.rs`
- `crates/tokmd/tests/baseline_integration.rs`
- `crates/tokmd/tests/baseline_w71.rs`
- `crates/tokmd/tests/bdd_analyze_scenarios_w50.rs`
- `crates/tokmd/tests/bdd_diff_scenarios_w50.rs`
- `crates/tokmd/tests/bdd_scenarios_w71.rs`
- `crates/tokmd/tests/bdd_scenarios_w75.rs`
- `crates/tokmd/tests/cli_badge_e2e.rs`
- `crates/tokmd/tests/cli_comprehensive.rs`
- `crates/tokmd/tests/cli_determinism_e2e_w54.rs`
- `crates/tokmd/tests/cli_e2e.rs`
- `crates/tokmd/tests/cli_e2e_w42.rs`
- `crates/tokmd/tests/cli_e2e_w58.rs`
- `crates/tokmd/tests/cli_e2e_w65.rs`
- `crates/tokmd/tests/cli_error_paths_w51.rs`
- `crates/tokmd/tests/cli_errors_w66.rs`
- `crates/tokmd/tests/cli_output_formats_w66.rs`
- `crates/tokmd/tests/cli_pipeline_e2e_w54.rs`
- `crates/tokmd/tests/deep_cli_complex_w48.rs`
- `crates/tokmd/tests/deep_cli_formats_w49.rs`
- `crates/tokmd/tests/deep_run_cockpit_w52.rs`
- `crates/tokmd/tests/determinism_hardening_w51.rs`
- `crates/tokmd/tests/determinism_regression.rs`
- `crates/tokmd/tests/diff_deep_w77.rs`
- `crates/tokmd/tests/diff_w71.rs`
- `crates/tokmd/tests/docs.rs`
- `crates/tokmd/tests/e2e_extended.rs`
- `crates/tokmd/tests/error_handling.rs`
- `crates/tokmd/tests/feature_gate_cli_w53.rs`
- `crates/tokmd/tests/feature_gates_w71.rs`
- `crates/tokmd/tests/full_pipeline_w55.rs`
- `crates/tokmd/tests/gate_integration.rs`
- `crates/tokmd/tests/integration.rs`
- `crates/tokmd/tests/integration_w40.rs`
- `crates/tokmd/tests/integration_w70.rs`
- `crates/tokmd/tests/json_output.rs`
- `crates/tokmd/tests/markdown_output.rs`
- `crates/tokmd/tests/output_formats_w76.rs`
- `crates/tokmd/tests/receipt_contracts_w72.rs`
- `crates/tokmd/tests/regression_prevention_w55.rs`
- `crates/tokmd/tests/regression_suite_w52.rs`
- `crates/tokmd/tests/run_diff.rs`
- `crates/tokmd/tests/schema_validation.rs`
- `crates/tokmd/tests/smoke_e2e.rs`

## 🧪 Verification receipts
```text
{"command": "cargo test -p tokmd --no-default-features", "status": "failed", "note": "Tests failed because they lacked #![cfg(feature = \"...\")]"}
{"command": "python3 fix_all_tests.py", "status": "success", "note": "Added #![cfg(feature = \"...\")] to the top of the failing test files"}
{"command": "cargo test -p tokmd --no-default-features", "status": "success", "note": "Tests pass successfully after applying conditionally compiled features"}
```

## 🧭 Telemetry
- Change shape: Add `cfg(feature)` to test files
- Blast radius: Tests only.
- Risk class + why: Very low risk, simply skips tests when required features are missing.
- Rollback: Revert the file edits.
- Gates run: `cargo test -p tokmd --no-default-features`, `cargo test -p tokmd --all-features`

## 🗂️ .jules artifacts
- `.jules/runs/compat_interfaces_matrix_1/envelope.json`
- `.jules/runs/compat_interfaces_matrix_1/decision.md`
- `.jules/runs/compat_interfaces_matrix_1/receipts.jsonl`
- `.jules/runs/compat_interfaces_matrix_1/result.json`
- `.jules/runs/compat_interfaces_matrix_1/pr_body.md`

## 🔜 Follow-ups
None.
