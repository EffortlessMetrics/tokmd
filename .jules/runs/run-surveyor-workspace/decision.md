# Analysis Complexity Seam Decision

Option A: Move `analyze_rust_function_complexity` into `tokmd-cockpit` (Recommended)
- What it is: Extract `crates/tokmd-analysis/src/source_complexity.rs` out of `tokmd-analysis` and move it directly into `tokmd-cockpit/src/gates/rust_source.rs` or a sibling module.
- Why it fits: The PR metrics cockpit is the *only* consumer of this heuristic. The architecture doc strictly forbids cross-tier mingling unless it's a shared contract. `tokmd-analysis` shouldn't own cockpit-specific gate heuristics. This cleans up the boundary.
- Trade-offs: Minor code move, but clarifies the architecture boundary.

Option B: Keep in `tokmd-analysis`, move `COCOMO81_COEFFICIENTS` to `tokmd-analysis-types`
- What it is: Move COCOMO constants.
- Why it fits: Also a boundary issue, but smaller.
- Trade-offs: `tokmd-analysis` already owns derived metrics, so keeping COCOMO there is mostly fine.

Decision: Option A. `tokmd-cockpit` relies on a very specific Rust AST heuristic that has nothing to do with the general `tokmd-analysis` presets. We will move the contents of `tokmd-analysis/src/source_complexity.rs` and its internal `mask.rs` over to `tokmd-cockpit`.
