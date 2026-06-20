## 🧭 Options considered

### Option A (recommended)
- Add a new integration test file `crates/tokmd/tests/bdd_error_paths_w71.rs` covering the edge case regression of error paths, specifically distinguishing unrecognized subcommands and missing paths.
- Fits the repo and shard because it improves scenario coverage around CLI interfaces.
- Trade-offs: Structure / Velocity / Governance. Adds testing overhead but ensures edge cases are covered.

### Option B
- Modify existing CLI tests to include BDD-style checks.
- When to choose: if we want to minimize new test files.
- Trade-offs: Makes existing tests longer and conflates BDD scenarios with component tests.

## ✅ Decision
We'll proceed with Option A to create a dedicated BDD test file `bdd_error_paths_w71.rs` which clearly separates scenario coverage for CLI error paths.
