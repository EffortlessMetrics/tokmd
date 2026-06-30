## 💡 Summary
Added seed corpora for `fuzz_toml_config` and `fuzz_policy_toml` fuzz targets. This improves fuzzing efficiency for complex parser inputs by providing valid starting shapes for the fuzzer to mutate.

## 🎯 Why
The `fuzz_toml_config` and `fuzz_policy_toml` targets parse significant and deeply structured external inputs (TOML). While `cargo fuzz` isn't fully operational in the sandbox, these targets lacked seed corpora, meaning any future fuzzer run would have to guess the schema from scratch, drastically slowing down the discovery of edge cases. Locking in valid shapes improves fuzzability and addresses the "corpus improvements that lock in real edge cases" target.

## 🔎 Evidence
- `fuzz/corpus/fuzz_toml_config/` and `fuzz/corpus/fuzz_policy_toml/` did not exist.
- `fuzz/fuzz_targets/fuzz_toml_config.rs` deserializes `TomlConfig` encompassing all CLI subcommands.
- `fuzz/fuzz_targets/fuzz_policy_toml.rs` parses complex `PolicyConfig` including ratchet rules and JSON pointers.
- Added seeds cover valid and complex structural shapes for both.

## 🧭 Options considered
### Option A (recommended)
- Add seed corpora to both targets.
- Fits the Fuzzer mission of "corpus improvements that lock in real edge cases" and directly improves the input hardening of the interface shard.
- Trade-offs: Structure is improved, future fuzzing Velocity increases, Governance risk is effectively zero since this is test/fuzz data only.

### Option B
- Extract a deterministic regression from a fuzzable surface into tests.
- When to choose it: If we had a specific known crash to lock in.
- Trade-offs: Existing tests are solid, and guessing regressions without fuzzer output is less productive than structurally improving the fuzzer's starting point.

## ✅ Decision
Option A. Adding the missing seed corpora provides immediate, deterministic improvements to the fuzzing infrastructure.

## 🧱 Changes made (SRP)
- Added `fuzz/corpus/fuzz_toml_config/seed_basic.toml` covering full configuration space.
- Added `fuzz/corpus/fuzz_toml_config/seed_advanced.toml` covering partial overrides.
- Added `fuzz/corpus/fuzz_toml_config/seed_types.toml` covering specific data boundaries.
- Added `fuzz/corpus/fuzz_policy_toml/seed_basic.toml` covering standard gate rules.
- Added `fuzz/corpus/fuzz_policy_toml/seed_complex.toml` covering ratchet rules and negative logic.

## 🧪 Verification receipts
```text
cargo check --workspace --all-targets --all-features
CI=true cargo test -p tokmd-settings -p tokmd-gate --verbose
```

## 🧭 Telemetry
- Change shape: add
- Blast radius: none (fuzz corpus only)
- Risk class: low (no production code changed)
- Rollback: `git clean -fd fuzz/corpus/`
- Gates run: check, test

## 🗂️ .jules artifacts
- `.jules/runs/fuzzer_input_hardening/envelope.json`
- `.jules/runs/fuzzer_input_hardening/decision.md`
- `.jules/runs/fuzzer_input_hardening/receipts.jsonl`
- `.jules/runs/fuzzer_input_hardening/result.json`
- `.jules/runs/fuzzer_input_hardening/pr_body.md`

## 🔜 Follow-ups
None.
