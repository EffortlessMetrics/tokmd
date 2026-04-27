## 💡 Summary
Added `#![cfg(feature = "analysis")]` to 52 integration tests that were failing when `tokmd` was tested with `--no-default-features`. This ensures the test suite passes gracefully in no-default-features mode while correctly covering these commands when features are enabled.

## 🎯 Why
Running `cargo test --no-default-features -p tokmd --tests` failed with over 100 test failures because the `analyze`, `badge`, `baseline`, `run`, and `gate` commands require the `analysis` feature. Integration tests covering these commands must be conditionally compiled to maintain matrix compatibility.

## 🔎 Evidence
Observed failure when testing `--no-default-features`:
```text
thread 'analyze_fun_preset_returns_eco_label' (22860) panicked at crates/tokmd/tests/analyze_integration.rs:140:5:
tokmd analyze failed: ExitStatus(unix_wait_status(256))
```

## 🧭 Options considered
### Option A (recommended)
- What it is: Add `#![cfg(feature = "analysis")]` to integration test files testing subcommands that depend on the `analysis` feature.
- Why it fits this repo and shard: It respects feature boundaries and matches the documented expectation for conditionally compiling test files in the `interfaces` shard.
- Trade-offs: Requires a broad file-level change, but perfectly aligns structure.

### Option B
- What it is: Ensure the CLI returns a graceful error code for missing features and have the tests expect that error code.
- When to choose it instead: When the CLI's intended behavior is to inform the user about feature requirements rather than just panic or return a generic error.
- Trade-offs: Pollutes test assertions with dual-mode (feature-on/feature-off) logic.

## ✅ Decision
Option A was chosen as it strictly isolates tests that mathematically cannot run when the `analysis` feature is omitted, keeping the test definitions clean.

## 🧱 Changes made (SRP)
- Added `#![cfg(feature = "analysis")]` to 52 test files in `crates/tokmd/tests/` that invoke subcommands requiring the `analysis` feature.

## 🧪 Verification receipts
```text
cargo test --no-default-features -p tokmd --tests
# Output:
# running 11 tests
# test check_ignore_nonexistent_path_reports_not_ignored ... ok
# test check_ignore_verbose_shows_detail ... ok
# test init_default_stderr_contains_template_hint ... ok
# test init_force_switches_template_on_existing_file ... ok
# test init_into_nonexistent_dir_fails_gracefully ... ok
# test init_non_default_template_omits_hint ... ok
# test init_print_node_contains_node_modules ... ok
# test init_print_rust_contains_target ... ok
# test init_refuses_overwrite_with_error_message ... ok
# test init_stderr_contains_initialized_message ... ok
# test init_stderr_contains_ready_message ... ok
#
# test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
#
# [...]
```

## 🧭 Telemetry
- Change shape: Cross-file addition of `#![cfg(feature = "analysis")]`.
- Blast radius: Internal test files only.
- Risk class: Low, only affects test compilation behavior in specific feature profiles.
- Rollback: Revert the file additions.
- Gates run: `cargo test --no-default-features -p tokmd --tests` and `cargo check --all-features -p tokmd`.

## 🗂️ .jules artifacts
- `.jules/runs/compat_interfaces_matrix/envelope.json`
- `.jules/runs/compat_interfaces_matrix/decision.md`
- `.jules/runs/compat_interfaces_matrix/result.json`
- `.jules/runs/compat_interfaces_matrix/pr_body.md`

## 🔜 Follow-ups
None.
