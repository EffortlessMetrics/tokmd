# API Stability in Performance Changes

**Context**: Attempted to batch `fs::metadata` calls in `tokmd-model` by introducing a `FileMetrics` map and passing it down to report functions.
**Pattern**: Breaking API change in a Tier 1 library crate for the sake of a performance improvement.
**Evidence**: PR closed with "changes tokmd-model public API... needs architectural review".
**Prevention**:
- **DO NOT** change public function signatures in library crates (especially `tokmd-model`, `tokmd-core`) without explicit architectural direction.
- If an optimization requires an API change, look for an alternative pattern (like bounded slice PRs) or memoization internal to the crate, even if slightly less "pure".
- Prefer optimizations completely contained within a single crate's internal functions or binary commands (`tokmd`).
