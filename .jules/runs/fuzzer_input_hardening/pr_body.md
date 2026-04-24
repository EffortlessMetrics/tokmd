## 💡 Summary
Extracted invariant tests from cargo-fuzz targets (`fuzz_toml_config.rs`, `fuzz_run_json.rs`, `fuzz_scan_args.rs`) into deterministic proptest suites in their respective crates (`tokmd-config`, `tokmd-core`, `tokmd-scan-args`). This satisfies the Fuzzer fallback gate by providing consistent, environment-independent parser coverage without relying on ASAN linkage.

## 🎯 Why
The `cargo-fuzz` toolchain fails to link locally due to missing ASAN symbols (`undefined symbol: __sancov_gen_...`). Rather than losing the invariant verification designed for these surfaces, we can fulfill the fallback gate profile explicitly by extracting these invariants into deterministic `proptest` suites that run seamlessly via `cargo test`.

## 🔎 Evidence
- **Finding**: Attempting to build or run fuzz targets locally via `cargo fuzz run fuzz_toml_config --features config` fails with linker errors.
- **Path**: `fuzz/fuzz_targets/fuzz_toml_config.rs`, `fuzz/fuzz_targets/fuzz_run_json.rs`, `fuzz/fuzz_targets/fuzz_scan_args.rs`.
- **Receipt**: Added deterministic fallback tests and verified via `cargo test -p tokmd-config --test proptest_toml_config_fuzz_fallback`, `cargo test -p tokmd-core --test proptest_run_json_fallback`, `cargo test -p tokmd-scan-args --test proptest_fuzz_fallback` (all passed).

## 🧭 Options considered
### Option A (recommended)
- **What it is**: Port the core invariant verification logic from `cargo-fuzz` targets into deterministic `proptest` test files directly inside the respective crates.
- **Why it fits**: Satisfies the explicit Fuzzer mission inline fallback: "If fuzz tooling is unavailable, record N/A and land deterministic proof work instead."
- **Trade-offs**:
  - **Structure**: Places deterministic hardening directly in the crate's test boundary, ensuring standard CI executes it on every PR.
  - **Velocity**: Mitigates the ASAN friction item by turning it into an immediate test win.
  - **Governance**: Fulfills the `fuzz` gate profile explicitly.

### Option B
- **What it is**: Abort the run and file a friction item about the ASAN linker failure.
- **When to choose it**: If creating deterministic fallbacks is not feasible.
- **Trade-offs**: Results in zero code/test improvements and creates unnecessary noise when a viable fallback is explicitly allowed.

## ✅ Decision
Option A. The logic encoded in the fuzz targets is valuable. Running it deterministically against thousands of properties via `proptest` achieves the same invariant verification goals without requiring specialized local setup.

## 🧱 Changes made (SRP)
- `crates/tokmd-config/tests/proptest_toml_config_fuzz_fallback.rs`: Ported `TomlConfig::parse` panic-resistance invariant.
- `crates/tokmd-core/tests/proptest_run_json_fallback.rs`: Ported `run_json` FFI panic-resistance and JSON response invariants.
- `crates/tokmd-scan-args/tests/proptest_fuzz_fallback.rs`: Ported path normalization, determinism, and ignore-flag invariants.

## 🧪 Verification receipts
```text
running 1 test
test toml_config_parse_never_panics ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.36s
```

```text
running 1 test
test run_json_never_panics ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.52s
```

```text
running 1 test
test scan_args_invariants ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.50s
```

## 🧭 Telemetry
- **Change shape**: Added tests
- **Blast radius**: None (tests only)
- **Risk class**: Lowest
- **Rollback**: `git rm crates/tokmd-config/tests/proptest_toml_config_fuzz_fallback.rs crates/tokmd-core/tests/proptest_run_json_fallback.rs crates/tokmd-scan-args/tests/proptest_fuzz_fallback.rs`
- **Gates run**: `cargo test` in `tokmd-config`, `tokmd-core`, `tokmd-scan-args`

## 🗂️ .jules artifacts
- `.jules/runs/fuzzer_input_hardening/envelope.json`
- `.jules/runs/fuzzer_input_hardening/decision.md`
- `.jules/runs/fuzzer_input_hardening/receipts.jsonl`
- `.jules/runs/fuzzer_input_hardening/result.json`
- `.jules/runs/fuzzer_input_hardening/pr_body.md`

## 🔜 Follow-ups
None.
