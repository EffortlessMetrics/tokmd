## 💡 Summary
Added deterministic property-based round-trip and rejection testing for missing config enum surfaces (`SensorFormat`, `NearDupScope`, `DiffRangeMode`) to improve input validation boundaries. The run originally attempted to fuzz parser surfaces via `cargo fuzz`, but libfuzzer ASAN integration in the test environment failed to link, prompting a fallback to deterministic tests.

## 🎯 Why
Several newly introduced CLI enum flags lacked comprehensive serialization round-trip property tests and negative (unknown rejection) tests. These parser edges benefit from rigorous deterministic constraints, especially since fuzzing toolchains are failing to compile in the CI/sandbox environment (`rust-lld: error: undefined symbol: __sancov_gen_.X`).

## 🔎 Evidence
Missing coverage was verified by inspecting `crates/tokmd-config/tests/properties.rs`. When compiling fuzz targets:
```
error: linking with `cc` failed: exit status: 1
...
rust-lld: error: undefined symbol: __sancov_gen_.242
```
Fuzzer gate expects: If fuzz tooling is unavailable, record N/A and land deterministic regression or harness commands instead.

## 🧭 Options considered
### Option A (recommended)
- what it is: Add deterministic property tests using `proptest` for the un-fuzzable or untested enum configurations (`SensorFormat`, `NearDupScope`, `DiffRangeMode`).
- why it fits this repo and shard: These enums are critical input validation surfaces within `tokmd-config`, matching the fuzzer persona's mission to harden inputs and add minimal harness improvements.
- trade-offs: Lower exploratory depth than true fuzzing, but deterministic and 100% reliable without flaky linker issues.

### Option B
- what it is: Fix the `cargo fuzz` linking errors by messing with build environments.
- when to choose it instead: If the prompt allows environment repair.
- trade-offs: High risk of time-boxing out without landing a PR-worthy patch due to obscure LLVM/rustc sanitizer mismatch.

## ✅ Decision
Option A. It explicitly satisfies the fallback constraint: "Otherwise land deterministic regressions or harness improvements instead of pseudo-fuzz claims."

## 🧱 Changes made (SRP)
- `crates/tokmd-config/tests/properties.rs`
  - Added `unknown_sensor_format_fails` property test
  - Added `unknown_near_dup_scope_fails` property test
  - Added `unknown_diff_range_mode_fails` property test
  - Added `sensor_format_roundtrip` macro test
  - Added `near_dup_scope_roundtrip` macro test
  - Added `diff_range_mode_roundtrip` macro test

## 🧪 Verification receipts
```
running 30 tests
test analysis_format_roundtrip ... ok
test badge_metric_roundtrip ... ok
test analysis_preset_roundtrip ... ok
test analysis_preset_kebab_case ... ok
test child_include_mode_roundtrip ... ok
test children_mode_roundtrip ... ok
test cockpit_format_display ... ok
test cockpit_format_serde_rt ... ok
test config_mode_roundtrip ... ok
test config_mode_serde_roundtrip ... ok
test diff_range_mode_roundtrip ... ok
test export_format_roundtrip ... ok
test handoff_preset_debug ... ok
test handoff_preset_serde_rt ... ok
test import_granularity_roundtrip ... ok
test init_profile_roundtrip ... ok
test near_dup_scope_roundtrip ... ok
test redact_mode_kebab_case ... ok
test sensor_format_roundtrip ... ok
test redact_mode_roundtrip ... ok
test shell_roundtrip ... ok
test table_format_kebab_case ... ok
test table_format_roundtrip ... ok
test unknown_cockpit_format_fails ... ok
test unknown_diff_range_mode_fails ... ok
test unknown_analysis_preset_fails ... ok
test unknown_handoff_preset_fails ... ok
test unknown_near_dup_scope_fails ... ok
test unknown_sensor_format_fails ... ok
test unknown_table_format_fails ... ok

test result: ok. 30 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
```

## 🧭 Telemetry
- Change shape: Test additions
- Blast radius: None (tests only)
- Risk class + why: Low (test-only change, purely additive)
- Rollback: Revert the PR
- Gates run: `cargo test -p tokmd-config`

## 🗂️ .jules artifacts
- `.jules/runs/run-1/envelope.json`
- `.jules/runs/run-1/receipts.jsonl`
- `.jules/runs/run-1/decision.md`
- `.jules/runs/run-1/result.json`
- `.jules/runs/run-1/pr_body.md`

## 🔜 Follow-ups
- Investigate ASAN/sancov linking failures for `cargo fuzz` targets on the current nightly/LLVM setup to unblock true fuzzing.
