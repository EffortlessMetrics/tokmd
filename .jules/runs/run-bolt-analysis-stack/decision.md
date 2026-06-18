# Decision

## Option A (Recommended)
Change `build_file_stats`, `build_max_file_report`, and `build_top_offenders` in `crates/tokmd-analysis/src/derived/files.rs` to process internal `FileStatRef` structs instead of creating full `FileStatRow`s upfront. This avoids performing expensive `.clone()` operations on path, module, and lang strings for every file in the codebase. Only the files that actually make it into the final Top Offenders or Max File reports will have their strings cloned into `FileStatRow`.

- Structure: Improves internal metrics aggregation logic without altering public APIs or outputs.
- Velocity: Faster for large codebases since String allocation is deferred.
- Governance: Maintain deterministic outputs while reducing allocations on the hot path of report generation.

## Option B
Do not optimize derived file metrics gathering, as these operations only run once per report. Instead look for other parsing caching mechanisms.

- Structure: Leaves existing code intact.
- Velocity: Suboptimal as String cloning is unnecessarily greedy on `O(n)` files for an `O(1)` slice of results.
- Governance: Complies, but leaves performance on the table.

## Decision
Option A. I have implemented Option A by introducing `FileStatRef<'a>` and deferring the `.clone()` behavior to a `to_owned()` method which is called selectively after sorting and filtering. The codebase compiles perfectly and all tests pass with identical outputs. The structural proof is solid (reduces string cloning allocations from `N` to `TOP_N * 5 + C`).
