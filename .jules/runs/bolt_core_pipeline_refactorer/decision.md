# Option A (recommended)

- **What it is:** Reorder the insertion logic in `crates/tokmd-model/src/lib.rs` for `collect_in_memory_file_rows` so that the parent file insertion happens after child files are inserted. This allows consuming the `path` and `module` variables, eliminating `.clone()` operations.
- **Why it fits:** Reduces unnecessary heap allocations on the hot path of memory file scanning, matching the `Bolt` performance refactoring goals.
- **Trade-offs:**
  - Structure: Minimal change to logic order, preserves the correctness since it populates a map.
  - Velocity: Extremely fast code change with immediate performance wins.
  - Governance: Zero risk of semantic alteration. Tests pass cleanly.

# Option B

- **What it is:** Optimize the `format!` allocations in `crates/tokmd-format/src/lib.rs` to use `write!` without intermediate string allocations.
- **Why to choose it instead:** It's another source of small performance wins during format serialization.
- **Trade-offs:** Requires introducing standard library `std::fmt::Write` trait into scopes, dealing with standard clippy lint rules about formatting empty new lines, which leads to bulkier logic and larger diffs, risking compilation stability without strong performance impact due to formatting being less of a hot path compared to memory data collection.
