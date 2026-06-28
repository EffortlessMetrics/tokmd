## 💡 Summary
Since `cargo fuzz` fails due to ASAN linking issues in the sandbox environment, I implemented deterministic property-based tests for `TomlConfig` to fulfill the Fuzzer persona fallback instruction. These tests guarantee parsing and roundtripping invariants (`JSON` and `TOML`) under arbitrary inputs.

## 🎯 Why
Fuzzing config/input surfaces is the primary goal, but environment blocks prevent `cargo fuzz` execution. Providing deterministic regression/property tests is the required fallback to harden the parsing surface.

## 🔎 Evidence
- file path(s): `crates/tokmd-settings/tests/properties_toml_config.rs`
- observed behavior / finding: Added deterministic tests mimicking fuzz coverage for `TomlConfig::parse` and serialization roundtrips.
- command receipt:
```text
cargo test -p tokmd-settings --test properties_toml_config
test toml_config_parses_valid_utf8 ... ok
test toml_config_json_roundtrip_prop ... ok
test toml_config_toml_roundtrip_prop ... ok
```

## 🧭 Options considered
### Option A (recommended)
- what it is: Implement deterministic property tests for `TomlConfig` config parsing.
- why it fits this repo and shard: Direct implementation of the fallback requirement for the `fuzz` gate profile when the environment blocks ASAN/libfuzzer execution. It ensures parser correctness and aligns with the `interfaces` shard.
- trade-offs: Structure / Velocity / Governance - Provides strong deterministic coverage without hallucinating fake fixes or changing CLI tools unnecessarily.

### Option B
- what it is: Document the ASAN blocker as a learning PR and do no work.
- when to choose it instead: If property testing the surface is unfeasible.
- trade-offs: Misses the opportunity to actively harden the configuration parsing using existing proptest infrastructure.

## ✅ Decision
Option A was chosen to fulfill the Fuzzer persona goals by hardening parser inputs using available deterministic property-testing tools.

## 🧱 Changes made (SRP)
- `crates/tokmd-settings/tests/properties_toml_config.rs`
- Fixed two unrelated clippy warnings in `crates/tokmd-model/src/rows.rs` and `crates/tokmd-analysis/src/api_surface/symbols/go.rs` to pass fallback gate checks.

## 🧪 Verification receipts
```text
cargo build --verbose
cargo test -p tokmd-settings --verbose
cargo xtask docs --check
cargo fmt -- --check
cargo clippy -- -D warnings
```

## 🧭 Telemetry
- Change shape: Property tests and minor lint fixes
- Blast radius: Testing and lints
- Risk class + why: Low, pure testing addition and syntax fixes for lints
- Rollback: Revert the PR
- Gates run: `cargo build`, `cargo test`, `cargo clippy`, `cargo fmt`, `cargo xtask docs --check`

## 🗂️ .jules artifacts
- `.jules/runs/run-fuzzer-1/envelope.json`
- `.jules/runs/run-fuzzer-1/decision.md`
- `.jules/runs/run-fuzzer-1/receipts.jsonl`
- `.jules/runs/run-fuzzer-1/result.json`
- `.jules/runs/run-fuzzer-1/pr_body.md`

## 🔜 Follow-ups
None
