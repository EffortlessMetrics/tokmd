---
id: cargo-xtask-gate-timeout
persona: Steward
style: Stabilizer
shard: tooling-governance
status: open
---

# `cargo xtask gate` consistently times out

**Context:** During release validation, attempting to run `cargo xtask gate` to verify quality gates fails due to consistent timeouts (>400s).

**Friction:** The command provisions temporary build targets in `/tmp/tokmd-gate-target-*` which exhaust time/disk resources or deadlock the CI/agent runner. This prevents automated full-suite validation via the canonical single command.

**Workaround:** Run the constituent checks manually:
```bash
cargo xtask version-consistency
cargo xtask docs --check
cargo xtask boundaries-check
cargo xtask publish-surface --verify-publish
cargo clippy -- -D warnings
cargo test -p xtask
```
Then clean up residual state:
```bash
rm -rf /tmp/tokmd-gate-target-*
```
