## 💡 Summary
Fixed documentation drift in `docs/SCHEMA.md` where the schema string identifiers were incorrectly documented with a `_VERSION` suffix.

## 🎯 Why
The documentation in `docs/SCHEMA.md` referred to `ENVELOPE_SCHEMA_VERSION` and `SENSOR_REPORT_SCHEMA_VERSION`. However, the actual identifiers in the Rust codebase are strings (e.g. `"sensor.report.v1"`), not integer versions. The correct constants in `tokmd-envelope` and `tokmd-analysis-types` are `SENSOR_REPORT_SCHEMA` and `ENVELOPE_SCHEMA`, without the `_VERSION` suffix. This drift could confuse contributors trying to use the ecosystem envelope schema identifiers.

## 🔎 Evidence
- `docs/SCHEMA.md` lines 97-100 contained incorrect constant names.
- `crates/tokmd-envelope/src/lib.rs` exports `pub const SENSOR_REPORT_SCHEMA: &str = "sensor.report.v1";`.
- `crates/tokmd-analysis-types/src/lib.rs` exports `pub const ENVELOPE_SCHEMA: &str = tokmd_envelope::SENSOR_REPORT_SCHEMA;`.

## 🧭 Options considered
### Option A (recommended)
- Update `docs/SCHEMA.md` to remove the `_VERSION` suffix from the schema string constants.
- Fits the `tooling-governance` shard and the Gatekeeper persona's mission to lock in deterministic output and protect contract-bearing surfaces from factual drift.
- Trade-offs: Zero velocity impact, improves governance structure.

### Option B
- Change the code constants to include `_VERSION`.
- When to choose: if the schema identifier was an integer version rather than a full semantic string.
- Trade-offs: Incorrect conceptually, breaks existing integrations expecting the current constant name.

## ✅ Decision
Option A. `docs/SCHEMA.md` has drifted and incorrectly added `_VERSION` to the string schema identifiers. Fixing the documentation aligns the contract docs with reality.

## 🧱 Changes made (SRP)
- `docs/SCHEMA.md`

## 🧪 Verification receipts
```text
cargo xtask docs --check (Documentation is up to date.)
cargo test -p tokmd-envelope (Passed)
cargo test -p tokmd-analysis-types (Passed)
```

## 🧭 Telemetry
- Change shape: Documentation patch
- Blast radius: docs
- Risk class: low (no behavior change)
- Rollback: git revert
- Gates run: `cargo xtask docs --check`, targeted `cargo test`

## 🗂️ .jules artifacts
- `.jules/runs/gatekeeper_contracts_run/envelope.json`
- `.jules/runs/gatekeeper_contracts_run/decision.md`
- `.jules/runs/gatekeeper_contracts_run/receipts.jsonl`
- `.jules/runs/gatekeeper_contracts_run/result.json`
- `.jules/runs/gatekeeper_contracts_run/pr_body.md`

## 🔜 Follow-ups
None.
