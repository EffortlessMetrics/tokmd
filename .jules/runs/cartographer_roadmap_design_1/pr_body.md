## 💡 Summary
Fixed a factual drift in `docs/requirements.md` where the `Analysis receipts` schema version incorrectly referenced `v5` instead of `v9`.

## 🎯 Why
The `docs/requirements.md` document listed `- **Analysis receipts**: Schema v5` under the `Receipt Contracts` section. However, `crates/tokmd-analysis-types/src/lib.rs` explicitly defines `ANALYSIS_SCHEMA_VERSION = 9`, and `docs/design.md` and `docs/architecture.md` correctly reference `v9`. This fixes the factual drift between the shipped reality and the requirements documentation.

## 🔎 Evidence
- **File**: `docs/requirements.md`
- **Observed behavior**: It claimed Analysis receipts were Schema v5.
- **Receipts**:
  - `grep -r "ANALYSIS_SCHEMA_VERSION =" crates/`
    - `crates/tokmd-analysis-types/src/lib.rs:pub const ANALYSIS_SCHEMA_VERSION: u32 = 9;`
  - `grep "Schema v" docs/requirements.md`
    - `- **Analysis receipts**: Schema v5`

## 🧭 Options considered
### Option A (recommended)
- Updating the schema version inline in `docs/requirements.md` from `v5` to `v9`.
- **Why it fits**: Directly addresses the factual drift within the tooling-governance shard.
- **Trade-offs**: High alignment with current repo structure and velocity, but could drift again in the future.

### Option B
- Removing the schema version number entirely from `docs/requirements.md` and pointing to `docs/design.md`.
- **When to choose**: If we want to centralize schema versions entirely to avoid future drifts.
- **Trade-offs**: Reduces immediate readability of the requirements doc.

## ✅ Decision
Chose Option A to accurately reflect the shipped `v9` schema in the requirements doc, keeping the document descriptive.

## 🧱 Changes made (SRP)
- `docs/requirements.md`: Updated `Schema v5` to `Schema v9` for Analysis receipts.

## 🧪 Verification receipts
```text
$ sed -i 's/- \*\*Analysis receipts\*\*: Schema v5/- \*\*Analysis receipts\*\*: Schema v9/' docs/requirements.md
$ cargo xtask docs --check
Documentation is up to date.
```

## 🧭 Telemetry
- **Change shape**: Docs update.
- **Blast radius**: `docs/` (Information/documentation only, no code change).
- **Risk class**: Trivial. Non-functional documentation fix.
- **Rollback**: Trivial revert.
- **Gates run**: `cargo xtask docs --check`

## 🗂️ .jules artifacts
- `.jules/runs/cartographer_roadmap_design_1/envelope.json`
- `.jules/runs/cartographer_roadmap_design_1/decision.md`
- `.jules/runs/cartographer_roadmap_design_1/receipts.jsonl`
- `.jules/runs/cartographer_roadmap_design_1/result.json`
- `.jules/runs/cartographer_roadmap_design_1/pr_body.md`

## 🔜 Follow-ups
None.
