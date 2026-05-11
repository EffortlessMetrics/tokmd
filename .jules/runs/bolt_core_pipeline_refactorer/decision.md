# Option A: Optimize `normalize_path`
- **What it is**: Refactor `normalize_path` in `crates/tokmd-model/src/lib.rs` to avoid redundant string clones and allocations when converting paths.
- **Why it fits**: Performance in the core pipeline shard. `normalize_path` is a hot-path function called for every file and child report during scanning/modeling.
- **Trade-offs**: Slightly more complex path logic, but significant allocation reduction on the hot path without changing determinism.

# Option B: Optimize BTreeMap String allocations in `rows.rs`
- **What it is**: Refactor the BTreeMap `Key` struct in `crates/tokmd-model/src/rows.rs` to use `&'a str` for the path as well, and/or avoid `.clone()` during child blob parsing by using shared references or `Cow`.
- **Why it fits**: Hot path work reduction.
- **Trade-offs**: Might be difficult due to lifetimes and strings owned by `FileRow`.

# Decision
I will choose **Option A**. The string allocations inside `normalize_path` when a prefix needs a trailing slash are unnecessarily allocating multiple `Cow::Owned` strings and cloning them. The optimized version avoids creating new `String`s for prefix matching when possible and just checks slices logically. This is a very clean Hot-path work reduction.
