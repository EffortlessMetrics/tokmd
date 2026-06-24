# Option A: Replace `.clone()` with `.into()` and borrowing in `diff.rs`

- **What it is**: Update `compute_diff_rows` and struct definition in `diff.rs` to minimize string allocations by replacing `.clone()` loops and unnecessary `cloned` conversions where lifetimes and string slice references can be appropriately used. We could map `&String` to `&str`, sort references, and only create owned strings when creating the `DiffRow`.
- **Why it fits this repo and shard**: Core data structures in `tokmd-types` and `tokmd-format` process numerous items; allocations add up. This adheres to the `perf-proof` gate profile requiring hot-path work reduction and unnecessary allocations reduction.
- **Trade-offs**:
    - *Structure*: Makes the codebase more idiomatic by removing redundant `.clone()` operations on strings.
    - *Velocity*: Moderate, since the scope is limited to formatting but requires matching lifetime bounds or making smaller targeted reductions.
    - *Governance*: Low risk; test suites will cover formatting correctness.

# Option B: Reduce intermediate buffer allocations in JSON formatting

- **What it is**: Refactor the output formatters to stream directly to `stdout`/writers rather than collecting data in intermediate data structures and running `.clone()` or `.to_string()`.
- **When to choose it instead**: If streaming outputs were a proven bottleneck and directly matching `tokmd_types` was simpler.
- **Trade-offs**: High impact but high complexity; involves touching multiple struct definitions and JSON ser/de paths which might break tests heavily.

# Decision

Proceeding with **Option A**. The low-hanging fruit in performance optimization here revolves around `String::clone()` in hot paths like `compute_diff_rows` inside `crates/tokmd-format/src/diff/compute.rs`. We can use `&str` to collect all unique languages instead of creating multiple cloned `String` copies just to sort and iterate.
