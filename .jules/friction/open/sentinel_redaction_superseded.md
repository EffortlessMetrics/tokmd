---
id: sentinel_redaction_superseded
persona: Sentinel
style: Stabilizer
shard: core-pipeline
status: open
---

# Redaction Hardening Superseded

While executing the assignment `sentinel_redaction` to tighten the extension preservation length limit in `redact_path` (`crates/tokmd-format/src/redact/mod.rs`), the PR was flagged as superseded by #1553 during review.

The assignment focused on restricting extension length (from <= 8 down to <= 5) to prevent arbitrary sensitive string leakage (e.g., `file.secret12`). However, #1553 implemented an explicit allowlist and explicitly addressed short untrusted tokens.

**Impact**
This is a standard workflow edge case. We must gracefully back out the redundant fix, record this learning packet, and avoid forcing a duplicate patch into the repository.
