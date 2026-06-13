# Decision

## Options Considered

### Option A: `sort_unstable_by` over `sort_by`
- **What it is:** Change `sort_by` to `sort_unstable_by` in `crates/tokmd-model/src/sorting.rs`.
- **Why it fits:** Performance optimization. `sort_unstable_by` is faster and allocates less than `sort_by`.
- **Trade-offs:** Output order of equal elements is not preserved. However, the sorting uses a `.then_with` to break ties based on a unique property (e.g. `lang`, `module`, `path`), making the sorting essentially deterministic even with `sort_unstable_by`.

### Option B: Pre-allocating string memory
- **What it is:** Replacing string clones with pre-allocating methods where appropriate.
- **Why it fits:** Small performance improvements in memory management.
- **Trade-offs:** Can complicate code for very small gain.

## Decision

I will go with **Option A**. The refactor reduces CPU cycles during aggregation sorting using unstable sort over stable sort, since the sort functions already provide a unique sorting criterion.
