## 💡 Summary
Added seed corpus files for `fuzz_toml_config` to bootstrap the fuzzer with valid and structurally invalid `tokmd.toml` examples. Included a deterministic test fallback since `cargo-fuzz` timed out in the CI environment.

## 🎯 Why
The `fuzz_toml_config` target lacked a seed corpus, forcing the fuzzer to learn TOML syntax and schema structure from scratch, leading to inefficient execution. Additionally, the fuzzer timed out on MSVC/current environments, so a deterministic regression test ensures the configuration schema invariants are continually verified in standard CI.

## 🔎 Evidence
- `crates/tokmd-settings/tests/deterministic_config.rs`
- Validated `TomlConfig::parse` with the kitchen sink of `tokmd.toml` parameters.
- `cargo test -p tokmd-settings --test deterministic_config` passed.

## 🧭 Options considered
### Option A (recommended)
- Add seed corpus and fallback deterministic tests.
- This directly addresses the input hardening instructions and `fuzz` gate profile expectations, balancing fuzzing efficiency with reliable CI fallback.
- Trade-offs: Structure (adds corpus files), Velocity (improves fuzzing), Governance (meets profile expectations).

### Option B
- Only add deterministic tests.
- This secures the CI but leaves future fuzzing inefficient when the toolchain is fixed.

## ✅ Decision
Pursued Option A to both accelerate fuzzing and guarantee deterministic coverage.

## 🧱 Changes made (SRP)
- Added `fuzz/corpus/fuzz_toml_config/seed_valid_kitchen_sink.toml`
- Added `fuzz/corpus/fuzz_toml_config/seed_invalid_types.toml`
- Added `crates/tokmd-settings/tests/deterministic_config.rs`

## 🧪 Verification receipts
```text
cargo test -p tokmd-settings --test deterministic_config
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## 🧭 Telemetry
- Change shape: Tests and corpus additions.
- Blast radius: None to production code.
- Risk class: Low, only adds tests.
- Rollback: Revert the added test files.
- Gates run: `cargo test`, `cargo fmt`, `cargo clippy`.

## 🗂️ .jules artifacts
- `.jules/runs/run-fuzzer-1/envelope.json`
- `.jules/runs/run-fuzzer-1/decision.md`
- `.jules/runs/run-fuzzer-1/receipts.jsonl`
- `.jules/runs/run-fuzzer-1/result.json`
- `.jules/runs/run-fuzzer-1/pr_body.md`

## 🔜 Follow-ups
None.
