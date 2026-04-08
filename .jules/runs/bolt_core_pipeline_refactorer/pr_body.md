## 💡 Summary
Optimized `tokmd-model` in-memory collection path by eliminating `String` clones for path and module variables.

## 🎯 Why
Memory allocations on paths scale linearly with the number of files. By avoiding two `.clone()` operations per scanned file, we can significantly reduce the memory overhead and allocation time during large-scale repository scans.

## 🔎 Evidence
Minimal proof:
- `crates/tokmd-model/src/lib.rs`
- Code logic consumed variables late, requiring clones on every iteration.
- Structurally validated that consuming the values last eliminates the need for any allocations.

## 🧭 Options considered
### Option A (recommended)
- what it is: Reorder the insertion logic in `crates/tokmd-model/src/lib.rs` for `collect_in_memory_file_rows` so that the parent file insertion happens after child files are inserted. This allows consuming the `path` and `module` variables, eliminating `.clone()` operations.
- why it fits this repo and shard: Directly improves the `tokmd-model` hot path memory profile safely.
- trade-offs: Minimal change to logic order, preserves the correctness since it populates a map. Zero risk of semantic alteration.

### Option B
- what it is: Optimize the `format!` allocations in `crates/tokmd-format` to use `write!`.
- when to choose it instead: If the application scales primarily on output formatting rendering rather than scanning.
- trade-offs: Slower to implement, large code perturbation, requires dealing with standard clippy lints about formatting string literals, and offers less bang for the buck than the memory scan optimization.

## ✅ Decision
Decided on Option A because it provides structurally guaranteed heap allocation reductions per file during the core model pipeline execution without modifying logic contracts.

## 🧱 Changes made (SRP)
- `crates/tokmd-model/src/lib.rs`: Moved the root `insert_row` call below the `children` loop, removing the `.clone()` method calls for `path` and `module`.

## 🧪 Verification receipts
```text
cargo test -p tokmd-model
cargo clippy -p tokmd-model -- -D warnings
```

## 🧭 Telemetry
- Change shape: Optimization
- Blast radius: `tokmd-model` internal data aggregator logic
- Risk class + why: Low, simply changes map insertion order where insertion order does not affect final sorting due to deterministic map/sort characteristics.
- Rollback: Revert the PR
- Gates run: Clippy, tests

## 🗂️ .jules artifacts
- `.jules/runs/bolt_core_pipeline_refactorer/envelope.json`
- `.jules/runs/bolt_core_pipeline_refactorer/decision.md`
- `.jules/runs/bolt_core_pipeline_refactorer/receipts.jsonl`
- `.jules/runs/bolt_core_pipeline_refactorer/result.json`
- `.jules/runs/bolt_core_pipeline_refactorer/pr_body.md`

## 🔜 Follow-ups
None.
