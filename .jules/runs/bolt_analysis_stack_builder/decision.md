# Decision

## Option A (recommended): Optimize sorting in `derived/files.rs` (Top Offenders)
- **What it is**: In `build_top_offenders` inside `crates/tokmd-analysis/src/derived/files.rs`, there are 5 full sorts (`sort_by`) of all files in the export data (which can be 100,000+ files for large repos). We only actually need the `TOP_N` (10) items from each. Rust's `slice::sort_unstable_by` is significantly faster than `slice::sort_by` (which allocates and does a merge sort), but even better, we can just use `select_nth_unstable_by(TOP_N, ...)` to partition the slice in O(N) time and then only sort the top 10 elements in O(1) time.
- **Why it fits**: We are asked to reduce hot-path work / compile-surface reductions / unnecessary work in the `analysis-stack` shard, specifically "performance work in analysis... prefer improvements that reduce repeated work". Sorting thousands of files 5 times is definitely a performance bottleneck for large codebases. The `select_nth_unstable_by` approach perfectly aligns with Target Ranking #1 (hot-path work reduction) and #4 (intermediate-buffer reduction).
- **Trade-offs**:
    - *Structure*: We keep the exact same interface. The output is deterministic because we still fully sort the top N elements.
    - *Velocity*: `select_nth_unstable_by` gives an order-of-magnitude speedup for large datasets compared to a full `sort_by`.
    - *Governance*: Negligible risk, standard Rust library feature.

## Option B: Optimize path tokenization in `topics/mod.rs`
- **What it is**: Change `tokenize_path` to avoid `replace('\\', "/").split('/')` and instead use `split(|c| c == '/' || c == '\\')`. Also swap the `stopwords` set from `BTreeSet` to `HashSet`.
- **When to choose it instead**: If the tokenization is the hottest part of the analysis.
- **Trade-offs**: While tokenization involves string allocations, a benchmark showed only minor improvements (1.05s -> 1.20s ? wait, the benchmark showed it was actually slightly slower or neutral). The `top_offenders` sorting change provides a much clearer algorithmic improvement from O(N log N) down to O(N).

## Decision
I will implement **Option A** to optimize `build_top_offenders` in `crates/tokmd-analysis/src/derived/files.rs`. It dramatically reduces repeated sorting work for large repos by taking advantage of `select_nth_unstable_by` to do O(N) selection followed by O(1) sorting of the top 10 elements.
