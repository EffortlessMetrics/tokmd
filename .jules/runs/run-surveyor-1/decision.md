## Option A (recommended)
Add the missing `Dockerfile.release` and `scripts/publish-release-crates.sh` entries to the `policy/non-rust-allowlist.toml` file.

*   **What it is**: The `cargo xtask check-file-policy` command fails because two non-Rust files are not tracked in the allowlist. Adding them explicitly resolves the issue and enforces file tracking policies.
*   **Why it fits**: The shard is `workspace-wide`, and the Surveyor persona focuses on workspace structure and boundary hygiene. The `non-rust-allowlist.toml` file governs the policy for all non-Rust files in the workspace. Fixing this drift improves the structural coherence and ensures the `check-file-policy` gate passes cleanly.
*   **Trade-offs**:
    *   **Structure**: Improves workspace policy coherence.
    *   **Velocity**: Trivial change; fixes a failing local/CI check, speeding up development.
    *   **Governance**: Restores the structural integrity of the workspace file tracking policy.

## Option B
Ignore the warning since `cargo xtask check-file-policy` runs in advisory mode by default.

*   **What it is**: Do nothing about the missing allowlist entries.
*   **When to choose it instead**: If the files were temporary or the policy file was about to be completely overhauled.
*   **Trade-offs**: Leaves broken policy rules in the workspace, risking future hard failures if `--strict` is enforced.

## Decision
**Option A** is chosen. It directly fixes a policy violation that affects workspace-wide hygiene and ensures `cargo xtask check-file-policy` passes cleanly in both advisory and strict modes. It is a highly focused and correct improvement for the Surveyor persona in the `workspace-wide` shard.
