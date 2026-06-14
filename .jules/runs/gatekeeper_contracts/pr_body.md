## 💡 Summary
Added `BASELINE_VERSION` and `SENSOR_REPORT_SCHEMA` to the workspace version bumping tool (`xtask bump`). This fixes drift during releases where schema versions for baselines and sensor reports could be missed.

## 🎯 Why
The `xtask bump` tool tracks known schema locations to increment them alongside `tokmd` updates, but it was missing the `BASELINE_VERSION` constant (used in `tokmd-analysis-types`) and `SENSOR_REPORT_SCHEMA` (used in `tokmd-envelope`). This gap posed a risk of version drift during publishing.

## 🔎 Evidence
- `xtask/src/tasks/bump.rs` did not include `BASELINE_VERSION` or `SENSOR_REPORT_SCHEMA` in its `SCHEMA_LOCATIONS` registry.
- Testing `cargo run -p xtask bump 1.13.2 --schema BASELINE_VERSION=2` previously failed with `Unknown schema constant`.

## 🧭 Options considered
### Option A (recommended)
- what it is: Add the missing schema constants to the `xtask` registry.
- why it fits this repo and shard: It protects contract-bearing surfaces (schema versions) via standard governance tooling (`tooling-governance`).
- trade-offs: Small maintainability cost in keeping `bump.rs` aligned with the codebase.

### Option B
- what it is: Update versions manually during releases.
- when to choose it instead: Never.
- trade-offs: High risk of drift and forgotten updates.

## ✅ Decision
Option A. It's an honest patch that closes a tooling gap in how we govern schema version bumps.

## 🧱 Changes made (SRP)
- `xtask/src/tasks/bump.rs`

## 🧪 Verification receipts
```text
cargo xtask bump 1.13.2 --schema BASELINE_VERSION=2 --schema SENSOR_REPORT_SCHEMA=2 --dry-run | tail -n 10

Schema version updates:
  - BASELINE_VERSION: 1 -> 2 (in crates/tokmd-analysis-types/src/baseline.rs)
  - SENSOR_REPORT_SCHEMA: 1 -> 2 (in crates/tokmd-envelope/src/lib.rs)

[DRY RUN] No changes written.
```

## 🧭 Telemetry
- Change shape: Tooling registry extension
- Blast radius: Internal release automation only
- Risk class + why: Very low; only affects local developer release flows.
- Rollback: Revert the commit.
- Gates run: `cargo test -p xtask`

## 🗂️ .jules artifacts
- `.jules/runs/gatekeeper_contracts/envelope.json`
- `.jules/runs/gatekeeper_contracts/decision.md`
- `.jules/runs/gatekeeper_contracts/receipts.jsonl`
- `.jules/runs/gatekeeper_contracts/result.json`
- `.jules/runs/gatekeeper_contracts/pr_body.md`

## 🔜 Follow-ups
None.
