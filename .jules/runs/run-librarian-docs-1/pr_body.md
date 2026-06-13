## 💡 Summary
Added executable doctests for core configuration structures in `tokmd-settings`. This proves configuration parsing behavior and keeps the interface documentation perfectly aligned with the schema.

## 🎯 Why
The `interfaces` shard requires high confidence in the public API contract, especially for configuration. Adding executable examples ensures that the `tokmd.toml` and legacy `config.json` parsing behavior remains factual and verifiable over time, preventing silent drift.

## 🔎 Evidence
- `crates/tokmd-settings/src/config.rs`
- `crates/tokmd-settings/src/profile.rs`
- Missing explicit executable coverage for top-level config struct parsing.

## 🧭 Options considered
### Option A (recommended)
- Add explicit `/// # Example` doctests to the core structs (`TomlConfig`, `ScanConfig`, `UserConfig`, `Profile`) demonstrating how they deserialize from configuration string literals.
- Why it fits: Aligns perfectly with the `Librarian` persona's goal of improving executable docs, reducing ambiguity around valid shapes.
- Trade-offs: Structure: High (locks in expectations). Velocity: Neutral. Governance: Low.

### Option B
- Build broad integration tests for config parsing.
- When to choose it: If edge cases or cross-cutting configuration behavior were the primary gap.
- Trade-offs: Less visible as documentation on the types themselves.

## ✅ Decision
Option A was selected to directly improve the executable examples on the public types, acting simultaneously as proof and documentation.

## 🧱 Changes made (SRP)
- `crates/tokmd-settings/src/config.rs`
- `crates/tokmd-settings/src/profile.rs`

## 🧪 Verification receipts
```text
running 5 tests
test crates/tokmd-settings/src/config.rs - config::ModuleConfig (line 98) ... ok
test crates/tokmd-settings/src/config.rs - config::ScanConfig (line 54) ... ok
test crates/tokmd-settings/src/config.rs - config::TomlConfig (line 11) ... ok
test crates/tokmd-settings/src/profile.rs - profile::Profile (line 41) ... ok
test crates/tokmd-settings/src/profile.rs - profile::UserConfig (line 16) ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## 🧭 Telemetry
- Change shape: Documentation and Proof improvements
- Blast radius: Only docs/tests; no production behavior changes.
- Risk class: Safe; strictly additive test coverage.
- Rollback: `git checkout crates/tokmd-settings/src/config.rs crates/tokmd-settings/src/profile.rs`
- Gates run: `cargo test -p tokmd-settings --doc`

## 🗂️ .jules artifacts
- `.jules/runs/run-librarian-docs-1/envelope.json`
- `.jules/runs/run-librarian-docs-1/decision.md`
- `.jules/runs/run-librarian-docs-1/receipts.jsonl`
- `.jules/runs/run-librarian-docs-1/result.json`
- `.jules/runs/run-librarian-docs-1/pr_body.md`

## 🔜 Follow-ups
None.
