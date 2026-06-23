# Decision

## Option A (recommended)
Move `tokei = { version = "14.0.0", default-features = false }` to `workspace.dependencies` in `Cargo.toml`, and update `crates/tokmd-scan` and `crates/tokmd-model` to use `tokei.workspace = true`. Similarly, move `ignore = "0.4.25"` to `workspace.dependencies` and update `crates/tokmd-scan` and `crates/tokmd-cockpit` to use `ignore.workspace = true`.
- **Why it fits**: Eliminates duplicate dependency declarations across multiple crates. This is a classic dependency hygiene improvement that matches the Auditor persona's goal of removing redundancy and tightening dependencies.
- **Trade-offs**:
  - *Structure*: Centralizes dependency management in the workspace `Cargo.toml`.
  - *Velocity*: Minor friction during the pull request, but pays off by preventing version mismatch in the future.
  - *Governance*: Aligns with the workspace-level `workspace.dependencies` strategy already used for `anyhow`, `blake3`, etc.

## Option B
Find an unused feature flag or unused dev-dependency and remove it.
- **Why it fits**: The prompt suggests we could remove an unused dependency. However, `tempfile` and `serde` are both actually used in the crates they are declared in.
- **When to choose it instead**: If we found a genuinely unused direct dependency.
- **Trade-offs**: Harder to prove unless we find one that is completely unused, which we didn't. Option A provides a clear, high-signal cleanup of redundant versions.

## ✅ Decision
Option A. I will move `tokei` and `ignore` dependency declarations to the workspace `Cargo.toml` and update the dependent crates (`tokmd-scan`, `tokmd-model`, and `tokmd-cockpit`) to use the workspace versions.
