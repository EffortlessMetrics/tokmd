# Decision

## Option A (recommended)
**Change BTreeMap to FxHashMap in near-duplicate pair building**

- **What it is**: Update `inverted_index` and `shared_fingerprint_counts` in `crates/tokmd-analysis/src/near_dup/pairs.rs` to use `rustc_hash::FxHashMap` instead of `std::collections::BTreeMap`.
- **Why it fits this repo and shard**: The benchmark shows that for both building the inverted index and counting the shared fingerprints, `FxHashMap` is significantly faster than `BTreeMap`. Specifically:
  - `inverted_index`: BTreeMap takes ~650µs, FxHashMap takes ~255µs (2.5x speedup)
  - `shared_counts`: BTreeMap takes ~3.25ms, FxHashMap takes ~340µs (9.5x speedup)
  This fits within the `analysis-stack` primary shard where `tokmd-analysis` resides, optimizing a core part of the `near_dup` analysis flow.
- **Trade-offs**:
  - Structure: We lose the ordered keys of `BTreeMap`, but since we only use these maps for grouping and looking up values, and not for ordered iteration, order doesn't matter for these two specific internal maps. The output of `build_pairs` still gets correctly sorted using `sort_pairs(&mut pairs)`.
  - Velocity: Simple internal change, no public API adjustments needed.
  - Governance: Uses `rustc_hash::FxHashMap` which is already a dependency in `tokmd-analysis` (used in `near_dup/fingerprint.rs`).

## Option B
**Change the pair loop nested loops**

- **What it is**: Pre-compute pair keys differently or avoid the `(usize, usize)` pair structure entirely.
- **When to choose it instead**: If replacing `BTreeMap` isn't fast enough.
- **Trade-offs**: More complex algorithmic changes might break determinism or alter the results slightly without immense speedups. The data structure swap is much safer and easier to prove correct.

## ✅ Decision
We choose **Option A**. The transition from `BTreeMap` to `FxHashMap` provides a substantial performance speedup to the pairing phase in `near-dup` analysis. The benchmark data clearly backs up this optimization and no deterministic output changes are expected because the final pair list is always explicitly sorted at the end.
