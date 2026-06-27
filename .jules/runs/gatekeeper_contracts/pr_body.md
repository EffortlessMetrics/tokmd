## 💡 Summary
Added missing `SCHEMA_LOCATIONS` (`BASELINE_VERSION` and `SENSOR_REPORT_SCHEMA`) to `xtask/src/tasks/bump.rs` to allow the workspace bump tool to manage them. Modified `bump.rs` to properly replace string schemas (like `SENSOR_REPORT_SCHEMA`) instead of just `u32` integers, fulfilling the memory directive and locking in determinism.

## 🎯 Why
The memory explicitly stated that new contract schema constants like `BASELINE_VERSION` and `SENSOR_REPORT_SCHEMA` must be registered in the `SCHEMA_LOCATIONS` array in `xtask/src/tasks/bump.rs` so they don't suffer version drift. Previously, `bump.rs` didn't track them, and its regex replacement only supported `u32` variables, making it fail on `&str` values.

## 🔎 Evidence
- `xtask/src/tasks/bump.rs` only handled `pub const {}: u32 = ...` replacements and lacked `BASELINE_VERSION` and `SENSOR_REPORT_SCHEMA` in its `SCHEMA_LOCATIONS` list.
- `crates/tokmd-envelope/src/lib.rs` has `pub const SENSOR_REPORT_SCHEMA: &str = "sensor.report.v1";`.
- I proved `cargo test -p xtask` works after my change, preventing test breakage.

## 🧭 Options considered
### Option A (recommended)
- Add `BASELINE_VERSION` and `SENSOR_REPORT_SCHEMA` to `SCHEMA_LOCATIONS` in `bump.rs`, and adjust string replacement logic to handle `&str` definitions.
- Fits the `tooling-governance` shard because it fixes workflow contract-determinism issues.
- Trade-offs: Requires a slight modification to the task string-replacement loop to support `&str` substitution natively.

### Option B
- Ignore `SENSOR_REPORT_SCHEMA` since it's a string, or migrate it to an integer.
- Trade-offs: Changing a major schema field type inside the contract might break dependents, while failing to track the string schema would leave it drifting.

## ✅ Decision
Option A. It explicitly fulfills the memory directive without breaking `SENSOR_REPORT_SCHEMA`'s expected `&str` format across downstream crates.

## 🧱 Changes made (SRP)
- `xtask/src/tasks/bump.rs`

## 🧪 Verification receipts
```text
$ cargo test -p xtask
test result: ok. 50 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 11.60s
$ cargo xtask docs --check
doc artifacts ok: 2 required doc(s), 54 family file(s), 1 active goal(s), 19 spec-index artifact(s), 0 spec-index lane(s)
$ cargo clippy -- -D warnings
Finished `dev` profile [unoptimized + debuginfo] target(s) in 32.32s
```

## 🧭 Telemetry
- Change shape: Workspace script update
- Blast radius: Internal release automation (workspace tooling/manifests)
- Risk class: Low - it only affects `cargo xtask bump --schema` which is manually invoked during releases.
- Rollback: `git revert`
- Gates run: `cargo test -p xtask`, `cargo xtask docs --check`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`

## 🗂️ .jules artifacts
- `.jules/runs/gatekeeper_contracts/envelope.json`
- `.jules/runs/gatekeeper_contracts/decision.md`
- `.jules/runs/gatekeeper_contracts/receipts.jsonl`
- `.jules/runs/gatekeeper_contracts/result.json`
- `.jules/runs/gatekeeper_contracts/pr_body.md`

## 🔜 Follow-ups
None.
