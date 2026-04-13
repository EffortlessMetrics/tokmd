## 💡 Summary
Added the missing `trend` property and `TrendComparison` schema types to `docs/SCHEMA.md` to match the generated JSON output of the `tokmd cockpit` receipt. The documentation now correctly reflects the struct layout implemented in `tokmd-types`.

## 🎯 Why
There was schema drift between the `CockpitReceipt` runtime struct in `crates/tokmd-types/src/cockpit.rs` and the documentation in `docs/SCHEMA.md`. The `trend` comparison object is present in `schema.json` and generated in Cockpit JSON receipts, but was completely missing from the manual Markdown schema documentation. Correcting this aligns the contract surfaces.

## 🔎 Evidence
- `docs/SCHEMA.md` was missing `trend` in the `CockpitReceipt` table and envelope example.
- `docs/schema.json` correctly defines the `trend` property with `$ref: "#/definitions/TrendComparison"`.
- `crates/tokmd-types/src/cockpit.rs` includes `pub trend: Option<TrendComparison>`.
- Verified fixing the schema docs ensures `cargo test -p tokmd --test docs` and `cargo test -p tokmd --test schema_doc_sync` still pass.

## 🧭 Options considered
### Option A (recommended)
- Update `docs/SCHEMA.md` to document the `trend` property, including the `TrendComparison` struct and sub-structs (`TrendMetric`, `TrendIndicator`), and update the Cockpit example snippet.
- This fits the `tooling-governance` shard by fixing contract drift in our documentation artifacts.
- Trade-offs: Structure is improved, Velocity remains high, Governance is enforced.

### Option B
- Modify only the `tokmd-types` crate to remove `trend`.
- When to choose it instead: If the `trend` metric was an experimental feature.
- Trade-offs: Would break existing baseline comparison functionality validated by `tokmd-cockpit` tests.

## ✅ Decision
Option A. The `TrendComparison` properties are fully verified via integration tests in `tokmd-cockpit`, and `trend` is already documented in `schema.json`. Updating `docs/SCHEMA.md` correctly aligns the documentation contract with the runtime outputs.

## 🧱 Changes made (SRP)
- `docs/SCHEMA.md`: Added `trend` to `CockpitReceipt` envelope JSON example.
- `docs/SCHEMA.md`: Added `trend` to `Cockpit Receipt Fields` table.
- `docs/SCHEMA.md`: Documented `Trend Comparison` (`trend`), `Trend Metric`, and `Trend Indicator` tables.
- `docs/SCHEMA.md`: Updated `Complete Cockpit Receipt Example` to include `"trend": null`.

## 🧪 Verification receipts
```text
{"command": "cargo test -p tokmd-types", "exit_code": 0}
{"command": "cargo test -p tokmd-cockpit", "exit_code": 0}
{"command": "cargo test -p tokmd-cockpit --test snapshot_w45 -- --nocapture", "exit_code": 0}
{"command": "cargo test -p tokmd --test docs", "exit_code": 0}
{"command": "cargo test -p tokmd --test docs_sync_w72", "exit_code": 0}
{"command": "cargo test -p tokmd --test schema_doc_sync", "exit_code": 0}
{"command": "cargo xtask gate --check", "exit_code": 0}
```

## 🧭 Telemetry
- Change shape: Schema documentation alignment.
- Blast radius: Only `docs/SCHEMA.md`.
- Risk class: Low risk. Purely additive to documentation to fix schema omission.
- Rollback: Revert the commit.
- Gates run: `cargo xtask gate --check`, `cargo test -p tokmd-types`, `cargo test -p tokmd-cockpit`, `cargo test -p tokmd --test schema_doc_sync`.

## 🗂️ .jules artifacts
- `.jules/runs/gatekeeper_contracts/envelope.json`
- `.jules/runs/gatekeeper_contracts/decision.md`
- `.jules/runs/gatekeeper_contracts/receipts.jsonl`
- `.jules/runs/gatekeeper_contracts/result.json`
- `.jules/runs/gatekeeper_contracts/pr_body.md`

## 🔜 Follow-ups
None.
