## Options considered

### Option A (recommended)
- **What it is**: Update the existing `no_run` doctest in `cockpit_workflow` to use a temporary git repository via `tempfile` and `std::process::Command`, ensuring deterministic, executable coverage for this critical API. Add a test demonstrating `try_parse_from` to `crates/tokmd/src/cli/parser.rs`.
- **Why it fits this repo and shard**: Aligns perfectly with the `docs-executable` gate profile requiring doctests to execute, avoiding silent drift in the `interfaces` shard.
- **Trade-offs**: Slightly increased doctest execution time since it invokes `git init/commit` in a temporary directory, but provides real proof that the API works.

### Option B
- **What it is**: Mock `tokmd-git` to simulate git behavior instead of actually running `git` inside the doctest.
- **When to choose it instead**: If the CI environment does not have `git` available or if running `git` becomes prohibitively slow across many tests.
- **Trade-offs**: Requires more complex setup, potentially modifying the `tokmd-git` API to be easily injectable or mockable just for doctests, increasing surface area.

## Decision
Chose Option A to create deterministic execution proofs while preserving the exact user-facing behavior in the doctest, fulfilling the Prover objective directly.
