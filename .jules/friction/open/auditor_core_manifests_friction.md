---
id: auditor_core_manifests_friction
persona: auditor
style: builder
shard: core-pipeline
status: open
---

# Friction Item

## Summary
Unable to find an unused dependency to remove in the `core-pipeline` shard (`tokmd-types`, `tokmd-scan`, `tokmd-model`, `tokmd-format`).

## Detail
`cargo machete --with-metadata` confirmed there are no unused dependencies in the target crates. While `cargo machete` identified unused dependencies in `tokmd-fuzz` (`anyhow`, `blake3`, `tempfile`) and `tokmd-node` (`napi-build`), these fall outside the assigned primary shard. Following the prompt's instruction to record out-of-shard targets as friction rather than chasing them, this learning PR captures the state and defers the out-of-shard cleanup.

## Impact
No direct harm, but highlights that the core pipeline crates are currently in a good state regarding direct dependency hygiene.

## Context
Prompt ID: `auditor_core_manifests`
