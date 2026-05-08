## 💡 Summary
Added a new proptest suite `scan_args_hardening` to `crates/tokmd/tests/properties.rs`. This deterministically verifies all input fuzzing invariants for `ScanArgs`, ensuring proper path normalization, determinism, and redaction boundary adherence, effectively capturing `fuzz_scan_args` logic without relying on fragile libfuzzer-sys environments.

## 🎯 Why
Local execution of `cargo fuzz run fuzz_scan_args` failed with `libfuzzer-sys` linking errors (`undefined symbol: __sancov_gen_`). Since the `fuzz` gate profile explicitly expects fallback to deterministic regressions or harness commands when fuzz tooling is unavailable, extracting these invariants into a core `proptest` locks in parser edge cases consistently across all CI environments.

## 🔎 Evidence
- `fuzz/fuzz_targets/fuzz_scan_args.rs` exists but failed to build.
- We ported its checks into deterministic property tests using `proptest` inside `crates/tokmd/tests/properties.rs`.
- Running `cargo test -p tokmd --test properties` verifies the new tests:
```text
test scan_args_hardening::scan_args_invariants ... ok
```

## 🧭 Options considered
### Option A (recommended)
- Extract the same deterministic invariant checks from `fuzz/fuzz_targets/fuzz_scan_args.rs` and port them to a `proptest` suite in `crates/tokmd/tests/properties.rs`.
- Why it fits: Fast, runs deterministically in CI, doesn't require nightly rustc or C sanitizers. Directly satisfies the "deterministic regression or harness commands" fallback gate expectation.
- Trade-offs:
  - Structure: Adds 80 lines of test code.
  - Velocity: Immediate unblocking vs waiting for C/LLVM linker fixes.
  - Governance: CI gets standard rust test coverage instantly.

### Option B
- Debug why `cargo fuzz` compilation of `fuzz_scan_args` fails with `undefined symbol: __sancov_gen_`.
- When to choose it: If we absolutely needed coverage-guided fuzzing and couldn't rely on random `proptest` inputs.
- Trade-offs: High risk of rabbit-holing into target-specific C/C++ linker flags, LLVM sanitizer flags, and nightly rustc bugs. Violates the instruction to not wait or get blocked.

## ✅ Decision
Option A was chosen to fulfill the deterministic proof expectation for the parser surface while bypassing environmental C/C++ linker blockers.

## 🧱 Changes made (SRP)
- Modified `crates/tokmd/tests/properties.rs` to append a new `scan_args_hardening` proptest module that mirrors `fuzz_scan_args.rs` invariants.

## 🧪 Verification receipts
```text
running 19 tests
test avg_function::zero_lines_returns_zero ... ok
test avg_function::exact_division ... ok
test avg_function::zero_files_returns_zero ... ok
test avg_function::rounds_correctly ... ok
test module_key::deterministic ... ok
test module_key::no_backslash_in_result ... ok
test module_key::matching_root_respects_depth ... ok
test module_key::never_crashes ... ok
test module_key::non_root_dir_returns_first_segment ... ok
test module_key::normalized_input_equivalence ... ok
test module_key::root_file_is_root ... ok
test path_normalization::idempotent ... ok
test path_normalization::always_forward_slash ... ok
test path_normalization::backslash_to_forward_preserves_segments ... ok
test path_normalization::never_crashes ... ok
test path_normalization::no_leading_slash ... ok
test scan_args_hardening::scan_args_invariants ... ok
test path_normalization::strips_leading_dot_slash ... ok
test path_normalization::prefix_stripping_works ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.34s
```

## 🧭 Telemetry
- Change shape: test additions
- Blast radius:
  - API: false
  - IO: false
  - Docs: false
  - Schema: false
  - Concurrency: false
  - Compatibility: false
  - Dependencies: false
- Risk class: low
- Risk explanation: Only modifies a test file (`properties.rs`), cannot affect runtime behavior.
- Rollback: Revert the commit adding the `scan_args_hardening` module.
- Gates run: `cargo fmt -- --check`, `cargo clippy -- -D warnings`, `cargo test -p tokmd --test properties`

## 🗂️ .jules artifacts
- `.jules/runs/fuzzer_input_hardening/envelope.json`
- `.jules/runs/fuzzer_input_hardening/receipts.jsonl`
- `.jules/runs/fuzzer_input_hardening/decision.md`
- `.jules/runs/fuzzer_input_hardening/result.json`
- `.jules/runs/fuzzer_input_hardening/pr_body.md`

## 🔜 Follow-ups
None
