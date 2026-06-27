## 💡 Summary
This is a learning PR. I explored the codebase looking for schema version drift to fix within the `core-pipeline` shard. I discovered that `BASELINE_VERSION` is missing from the workspace schema bump task (`xtask/src/tasks/bump.rs`), but since `xtask` is outside my allowed paths, I have recorded this as a friction item instead of chasing the fix out-of-bounds.

## 🎯 Why
The assignment was to protect contract-bearing surfaces. The strongest target found was schema drift in the workspace bump script. Because `xtask` is strictly outside the `core-pipeline` shard boundaries, making the fix directly would violate shard discipline. The correct procedure is to surface the drift as friction and exit gracefully as a learning PR.

## 🔎 Evidence
- **File**: `crates/tokmd-analysis-types/src/baseline.rs`
- **Finding**: The constant `BASELINE_VERSION` is defined here but missing from `SCHEMA_LOCATIONS` in `xtask/src/tasks/bump.rs`.
- **Receipt**:
```text
$ grep -rn "BASELINE_VERSION" crates/tokmd-analysis-types/src/baseline.rs
20:pub const BASELINE_VERSION: u32 = 1;
```

## 🧭 Options considered
### Option A (recommended)
- Record a friction item describing the schema drift and file a learning PR.
- **Why it fits**: Obeys strict shard discipline. Acknowledges the problem without violating architectural boundaries.
- **Trade-offs**: Structure (high) / Velocity (medium) / Governance (high).

### Option B
- Modify `xtask/src/tasks/bump.rs` directly.
- **When to choose**: When the `xtask` directory is explicitly added to the allowed shard paths.
- **Trade-offs**: Violates prompt constraints and risks corrupting tools outside the assigned domain.

## ✅ Decision
**Option A**. Sticking to the assigned shard is a hard constraint. The friction item properly documents the gap so a future run assigned to `xtask` can safely apply the fix.

## 🧱 Changes made (SRP)
- Created `.jules/friction/open/FRIC-20240615-001.md`.
- No production files were changed.

## 🧪 Verification receipts
```text
$ grep -rn "BASELINE_VERSION" crates/tokmd-analysis-types/src/baseline.rs
20:pub const BASELINE_VERSION: u32 = 1;
$ grep -rn "SCHEMA_LOCATIONS" xtask/src/tasks/bump.rs
30:const SCHEMA_LOCATIONS: &[SchemaVersionLocation] = &[
```

## 🧭 Telemetry
- **Change shape**: Learning PR.
- **Blast radius**: None.
- **Risk class**: Trivial. Only per-run artifacts and friction items.
- **Rollback**: None.
- **Gates run**: None.

## 🗂️ .jules artifacts
- `.jules/runs/gatekeeper_determinism/envelope.json`
- `.jules/runs/gatekeeper_determinism/decision.md`
- `.jules/runs/gatekeeper_determinism/receipts.jsonl`
- `.jules/runs/gatekeeper_determinism/result.json`
- `.jules/runs/gatekeeper_determinism/pr_body.md`
- `.jules/friction/open/FRIC-20240615-001.md`

## 🔜 Follow-ups
- A future run assigned to the workspace tooling shard should address `FRIC-20240615-001`.
