# Option A
Fix the missed mutants in `is_default_policy` inside `tokmd-types` by adding assertions for `InclusionPolicy::Summary` and `InclusionPolicy::HeadTail`. This directly targets the `tokmd-types` crate and mutation gaps.

# Option B
Continue investigating `tokmd-model` or `tokmd-scan` for mutation test gaps. However, due to timeout constraints and large build times, it is inefficient to find mutation test improvements in `tokmd-model` and wait for them to finish.

**Decision**: Option A. It's direct, actionable, closes the cargo mutants gap in `tokmd-types/src/lib.rs` for `is_default_policy`, and avoids the huge test timeouts present in `tokmd-model`.
