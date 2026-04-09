# Surveyor Workspace Decision

## Option A
Complete the split described in `tokmd-config/src/lib.rs` by renaming `tokmd-config` to `tokmd-cli` and updating all workspace dependencies.
**Why it fits:** The comment in `tokmd-config/src/lib.rs` explicitly mentions:
`Future Direction: Split into tokmd-settings (pure config) and tokmd-cli (Clap parsing)`. `tokmd-settings` already exists, but `tokmd-config` was left named as `tokmd-config`.
**Trade-offs: Structure / Velocity / Governance:**
Structure: Improves crate naming coherence and accurately reflects the crate's purpose (Clap CLI parsing).
Velocity: Small hit from updating dependencies, but better long-term navigation.
Governance: Adheres to the established architecture plan.

## Option B
Keep `tokmd-config` as is, but move all remaining pure config structs into `tokmd-settings`.
**Why it fits:** Would cleanly separate CLI vs Config logic without renaming the crate.
**Trade-offs:** `tokmd-config` is a misnomer if it only does CLI parsing, which `tokmd-cli` better describes.

## ✅ Decision
I will pursue Option A. I will rename the crate `tokmd-config` to `tokmd-cli` (or similar appropriate name, perhaps keeping the folder `crates/tokmd-cli` or just renaming the crate name in `Cargo.toml`). Wait, let me check `crates/tokmd/Cargo.toml` and where `tokmd-config` is used. Renaming a crate that holds CLI argument definitions to `tokmd-cli` perfectly aligns with the `Future Direction` comment.
