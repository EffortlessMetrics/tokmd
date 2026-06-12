## Option A: Consolidate `toml` dependency into `[workspace.dependencies]`
- What it is: Move `toml = "1.1.2"` to the root `Cargo.toml` under `[workspace.dependencies]`, and update `tokmd`, `tokmd-gate`, and `tokmd-settings` to use `{ workspace = true }`.
- Why it fits this repo and shard: It aligns with "dependency hygiene improvements" and "metadata alignment", fitting the Steward persona's goals for release/governance. This centralizes versioning for `toml` like `serde` and `anyhow`.
- Trade-offs: Structure is improved, no velocity impact, better governance.

## Option B: Fix the case-insensitive paths collision (if real)
- Let me check if there's an actual collision in the workspace.
