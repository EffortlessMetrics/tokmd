---
id: steward-gate-timeout
persona: Steward
style: Stabilizer
shard: tooling-governance
status: open
---

# `cargo xtask gate` timeout issue

**Context:** While verifying the release and governance surfaces, I ran `cargo xtask gate`.

**Friction:** The `cargo xtask gate` command creates temporary directories in `/tmp/` and can silently time out after 400s+ without producing a clear error message as to why.

**Workaround:** Run the constituent checks manually (`cargo xtask version-consistency`, `cargo xtask docs --check`, `cargo fmt -- --check`, etc) and clean up the `/tmp/tokmd-gate-target-*` directories manually.
