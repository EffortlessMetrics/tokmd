## 💡 Summary
Updated the `fuzz_toml_config` target and `fuzz/Cargo.toml` to depend on `tokmd_settings` instead of `tokmd_config`. This fixes the fuzz target build which was broken when `TomlConfig` moved to `tokmd_settings` during a recent refactor.

## 🎯 Why
The `tokmd-fuzz` crate failed to compile the `fuzz_toml_config` target because it was looking for `TomlConfig` in `tokmd-config`, which no longer exports it after the refactor to `tokmd-settings`. This left the fuzzing corpus and configuration parsing logic without functioning fuzz coverage.

## 🔎 Evidence
- `fuzz/fuzz_targets/fuzz_toml_config.rs`
- Attempting to compile the fuzz target resulted in unresolved imports.
```text
error: failed to resolve use tokmd_config::TomlConfig
```

## 🧭 Options considered
### Option A (recommended)
- Update the `fuzz_toml_config` fuzz target and the corresponding `config` feature in `fuzz/Cargo.toml` to depend on `tokmd_settings`.
- Restores the fuzz harness for TOML configuration parsing.
- Trade-offs: Structure / Velocity / Governance: Improves velocity by unbreaking the fuzz harness; maintains structure by adhering to the recent crate refactor.

### Option B
- Delete the fuzz target or exclude it from the workspace manually.
- Use this when the target is no longer relevant.
- Trade-offs: Reduces test coverage and governance over configuration parsing.

## ✅ Decision
Option A. It directly addresses the issue, unbreaks the parser fuzzability, and restores deterministic regression checks for configuration inputs.

## 🧱 Changes made (SRP)
- `fuzz/Cargo.toml`: Replaced `tokmd-config` with `tokmd-settings` for the `config` feature.
- `fuzz/fuzz_targets/fuzz_toml_config.rs`: Replaced `use tokmd_config::TomlConfig` with `use tokmd_settings::TomlConfig`.

## 🧪 Verification receipts
```text
cargo check -p tokmd-fuzz --features config
cargo check -p tokmd-fuzz --all-features
Finished `dev` profile [unoptimized + debuginfo] target(s) in 23.45s
```

## 🧭 Telemetry
- Change shape: Dependency update and import change
- Blast radius: Fuzz tooling / tests only
- Risk class: Low
- Rollback: Revert the commit
- Gates run: cargo test -p tokmd-config, cargo test -p tokmd-settings, cargo check -p tokmd-fuzz

## 🗂️ .jules artifacts
- `.jules/runs/run-fuzzer-01/envelope.json`
- `.jules/runs/run-fuzzer-01/decision.md`
- `.jules/runs/run-fuzzer-01/receipts.jsonl`
- `.jules/runs/run-fuzzer-01/result.json`
- `.jules/runs/run-fuzzer-01/pr_body.md`
- `.jules/friction/open/tokmd_python_clippy.md`
- `.jules/friction/open/tokmd_context_git_clippy.md`

## 🔜 Follow-ups
Filed friction items for failing workspace clippy checks in `tokmd-python` and `tokmd-context-git`.
