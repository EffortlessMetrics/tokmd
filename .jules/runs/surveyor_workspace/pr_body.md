## 💡 Summary
Removed the dead `tokmd-config` microcrate from the workspace completely, as it was merely a compatibility facade from an earlier split. Migrated its extensive integration test suite into `tokmd-settings` to preserve coverage. Also removed unused dependencies (`anyhow`, `blake3`) from the `fuzz` crate.

## 🎯 Why
Keeping empty crates like `tokmd-config` complicates the dependency graph and wastes `cargo build` and review bandwidth. It was acting as a dummy tier-4 dependency in `xtask` boundary tests, which is better served by the real tier-4 crate, `tokmd-settings`. The `fuzz` crate dependencies were noted in an open friction item as unused.

## 🔎 Evidence
- `tokmd-config/src/lib.rs` was a single file re-exporting `tokmd::cli`.
- `cargo machete` flagged `anyhow` and `blake3` as unused in `fuzz/Cargo.toml`.
- Replaced the boundaries check target to `tokmd-settings` in `xtask/tests/xtask_deep_w74.rs` and others.

## 🧭 Options considered
### Option A (recommended)
- Remove `tokmd-config` entirely, migrate tests to `tokmd-settings`, point `xtask` boundaries tests at `tokmd-settings`, and remove `tokmd-fuzz` deps.
- **Why it fits**: Simplifies the workspace architecture and crate graph, fulfilling the Surveyor mission.
- **Trade-offs**: Structure improves by dropping a crate; Velocity is unchanged; Governance improves by closing out an old migration.

### Option B
- Leave `tokmd-config` as-is and just submit a learning PR.
- **Trade-offs**: Leaves a dead crate in the repository permanently and leaves the friction item open.

## ✅ Decision
Option A. It's a clean deletion of a dead crate and resolves unused dependencies, perfectly fitting the Surveyor's mandate.

## 🧱 Changes made (SRP)
- Deleted `crates/tokmd-config/`
- Copied `crates/tokmd-config/tests/*.rs` to `crates/tokmd-settings/tests/tokmd_config_migrated/` and updated `use` statements.
- Updated `xtask/src/tasks/boundaries_check.rs` and `xtask/tests/*.rs` to replace `tokmd-config` with `tokmd-settings` in the `FORBIDDEN` list.
- Removed `anyhow` and `blake3` from `fuzz/Cargo.toml` (`[dependencies]` and `content` feature).

## 🧪 Verification receipts
```text
cargo machete
cargo test -p xtask
cargo test -p tokmd-settings
cargo test --workspace
```

## 🧭 Telemetry
- Change shape: Deletion & test migration
- Blast radius: Low. The crate was already out of the workspace default members and unused by `tokmd-core`.
- Risk class: Low
- Rollback: `git revert`
- Gates run: `cargo xtask docs --check`, `cargo xtask version-consistency`, `cargo test --workspace`.

## 🗂️ .jules artifacts
- `.jules/runs/surveyor_workspace/envelope.json`
- `.jules/runs/surveyor_workspace/decision.md`
- `.jules/runs/surveyor_workspace/receipts.jsonl`
- `.jules/runs/surveyor_workspace/result.json`
- `.jules/runs/surveyor_workspace/pr_body.md`

## 🔜 Follow-ups
None.
