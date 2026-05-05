# Decision Record

## Option A
Fix `is_test_path` in `tokmd-analysis-types` to uniformly normalize paths by prepending a `/` prior to checking for directory boundary substring patterns.
- Fits the repo and shard by reducing uncertainty around path invariants. Previously, if a file was situated directly at a repository root as `test/foo.rs`, the logic checking for `/test/` failed to mark it as a test. Ensuring paths logically start with a slash resolves this cleanly.
- **Trade-offs:**
  - Structure: Preserves existing logic overall.
  - Velocity: Quick, low-risk fix.
  - Governance: High alignment with analysis accuracy goals.

## Option B
Re-write `is_test_path` to split the path into segments and iterate over segments instead of doing string allocations and sub-string matching on a prepended string.
- When to choose it: Only if performance of string allocation on test path resolution proves to be a hot-path bottleneck.
- **Trade-offs:**
  - Could be marginally faster, but introduces more branching logic and diverges significantly from the existing stable behavior, risking regressions in edge cases.

## Decision
**Option A** is chosen. It fixes a proven gap where boundary conditions (a test folder acting as a path root) failed test inference.
