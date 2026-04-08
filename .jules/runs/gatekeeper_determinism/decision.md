# Decision

## Option A (recommended)
Do not modify code. Create a learning PR to document that the core pipeline already enforces explicit BTreeMap sorting correctly.
- **What it is:** Instead of modifying code out-of-bounds, generate a learning PR detailing that `crates/tokmd-analysis-content` is outside the allowed `core-pipeline` shard, but lacks explicit multi-key BTreeMap sorting for deterministic serialization. Record this as an open friction item.
- **Why it fits this repo and this shard:** It strictly adheres to the Gatekeeper persona constraints ("If the strongest target you find is outside the shard, record it as friction instead of chasing it.").
- **Trade-offs:** Slower immediate fix for the analysis formatting out-of-shard, but correctly follows system policy.

## Option B
Fix the formatting bug out of bounds anyway.
- **When to choose it instead:** When the policy allows cross-shard operations.
- **Trade-offs:** Fails to comply with strict prompt instructions.

## ✅ Decision
Option A. Create a learning PR and record the missing deterministic sorts as friction for the `Surveyor` or `Specsmith` persona depending on the target shard assignment.
