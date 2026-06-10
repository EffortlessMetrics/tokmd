# Decision

## Option A: Replace BTreeMap with FxHashMap in High-Frequency Paths
- What it is: Replaces `BTreeMap` with `FxHashMap` in computationally intensive analysis areas like `content/mod.rs` (for duplicate reporting), `git/mod.rs` (for freshness reporting), and `api_surface/report.rs` (for API surface aggregation).
- Why it fits: BTreeMap is heavily used for intermediate aggregation (e.g., mapping hashes to files, module paths to code bytes) where ordering is often only required at the very end of the function before returning the report. Since we're dealing with string keys and high iteration counts in analysis algorithms, using a faster hasher (`rustc_hash::FxHashMap`) significantly speeds up analysis and reduces unnecessary allocation overhead.
- Trade-offs: Requires a dependency addition (`rustc-hash`) to the optional features in `Cargo.toml`. Also requires sorting the maps manually if a deterministic result order is required (which the codebase already does via `.sort_by()`).

## Option B: Optimize Clone Usage
- What it is: Reduces `clone()` calls on `String` values by modifying how references are handled during intermediate aggregation, particularly in module/path comparisons.
- When to choose: If adding `FxHashMap` is considered too risky, or if cloning represents the sole bottleneck.
- Trade-offs: Harder to implement securely without altering function contracts. Often gives lower ROI than directly changing the underlying data structure powering the aggregations.

## Decision
I'm going with Option A. High-frequency allocations and tree rebalances for intermediate data gathering are classic performance bottlenecks. Rust's default `BTreeMap` is great when continuous sorting is needed, but for simple counts/aggregations before a final `sort_by`, `FxHashMap` is drastically faster. Since determinism is still maintained by sorting the final reports, this is a clear win. I'll replace it in three specific modules: `api_surface/report.rs`, `content/mod.rs`, and `git/mod.rs`, and add the `rustc-hash` dependency.
