---
id: superseded_redaction_patch
persona: Sentinel 🛡️
style: Stabilizer
shard: core-pipeline
status: open
---

# Superseded Redaction Patch

During a run attempting to harden path redaction (using an explicit extension allowlist in `tokmd-format/src/redact/mod.rs`), the PR was found to be superseded by PR #1553, which successfully merged a salvaged version of the allowlist policy and test updates without the stale generated payload issues encountered in this execution.

This is documented to comply with the Sentinel learning PR policy when an intended fix is superseded by another merged PR.
