## 💡 Summary
Renamed `tokmd-config` to `tokmd-cli-args` to correctly reflect its specific purpose of Clap parsing and CLI arguments, fulfilling the crate split previously mentioned in the codebase.

## 🎯 Why
The `tokmd-config` crate's name was misleading because `tokmd-settings` now houses the raw/pure configuration structs. As noted in the `Future Direction` section in `crates/tokmd-config/src/lib.rs`, the goal was to separate pure config into `tokmd-settings` and CLI parsing into a dedicated crate. Renaming `tokmd-config` to `tokmd-cli-args` correctly finalizes this structural intent and aligns the crate names with their actual functionality.

## 🔎 Evidence
- `crates/tokmd-config/src/lib.rs` (before changes) had a section `//! ## Future Direction` stating: `* Split into tokmd-settings (pure config) and tokmd-cli (Clap parsing)`.
- `tokmd-settings` was already created but `tokmd-config` still held the Clap args and had not been renamed.

## 🧭 Options considered
### Option A (recommended)
- Rename `tokmd-config` to `tokmd-cli-args` and update all workspace dependencies and imports.
- It fits the repo/shard by directly addressing a stated architectural direction for boundary hygiene in `tokmd-config/src/lib.rs`.
- Trade-offs: Structure is improved, matching names to logic. Velocity is slightly impacted for consumers needing to update crate names, but governance and navigation in the codebase become cleaner.

### Option B
- Keep `tokmd-config` as is and remove `tokmd-settings` completely, folding its structs back in.
- Choose this if we prefer monolithic configuration crates.
- Trade-offs: This regresses against the existing split and bundles CLI-specific `clap` dependencies into contexts that only need pure data structs, bloating dependency trees.

## ✅ Decision
Option A was chosen to fulfill the documented `Future Direction` comment and properly establish the crate boundary between pure TOML settings and CLI arguments.

## 🧱 Changes made (SRP)
- Renamed the directory `crates/tokmd-config` to `crates/tokmd-cli-args`.
- Renamed the crate to `tokmd-cli-args` in its `Cargo.toml`.
- Updated all workspace `Cargo.toml` and `fuzz/Cargo.toml`, `xtask/Cargo.toml` references from `tokmd-config` to `tokmd-cli-args`.
- Replaced all Rust module imports (`tokmd_config`) with `tokmd_cli_args`.
- Updated documentation references in `docs/architecture.md`, `docs/implementation-plan.md`, and `docs/testing.md`.

## 🧪 Verification receipts
```text
$ cargo check --workspace --all-targets --all-features
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.02s
```

## 🧭 Telemetry
- Change shape: Crate renaming and dependency refactoring.
- Blast radius: API (internal module renaming), dependencies (updates `Cargo.toml` refs).
- Risk class: Low, only changes crate and internal import names; behavior is identical.
- Rollback: Revert the git commit renaming the folder and references.
- Gates run: `cargo check`

## 🗂️ .jules artifacts
- `.jules/runs/surveyor_workspace/envelope.json`
- `.jules/runs/surveyor_workspace/decision.md`
- `.jules/runs/surveyor_workspace/receipts.jsonl`
- `.jules/runs/surveyor_workspace/result.json`
- `.jules/runs/surveyor_workspace/pr_body.md`

## 🔜 Follow-ups
None.
