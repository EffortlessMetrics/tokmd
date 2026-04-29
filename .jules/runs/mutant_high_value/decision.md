# Decision

## Option A (recommended)
Add tests to cover mutant gaps in `normalize_path` within `tokmd-model`.
The function `normalize_path` contains optimized "fast path" vs "slow path" logic for stripping path prefixes.
Cargo mutants revealed that we were missing tests for:
1. `normalize_path_prefix_partial_match`: Ensuring a prefix doesn't match partially against directory names (`project` vs `project_extra`). This prevents bugs where dropping a trailing slash check erroneously strips partial matches.
2. `normalize_path_prefix_mixed_slashes`: Ensuring the fast path correctly bails out when the prefix contains backslashes but ends with a forward slash.

### Trade-offs: Structure / Velocity / Governance
* Structure: Low overhead. Adding BDD test cases ensures these specific invariants are preserved.
* Velocity: Minimal impact on CI time.
* Governance: Directly aligns with the "Mutant" persona's goal of closing concrete missed-mutant gaps in core models.

## Option B
Test mutant gaps in other crates (e.g. `tokmd-types` or `tokmd-scan`).
Wait for the full mutant run on the other crates.

### Trade-offs: Structure / Velocity / Governance
* Structure: Potentially more complex if the gaps require more context.
* Velocity: Slower as we have to wait for the cargo mutants run to finish on other crates, which takes 5-10+ minutes.
* Governance: Does not directly address the gaps we already found in `tokmd-model`.

## Decision
Choose **Option A**. The `normalize_path` function is a core utility that determines the paths for all reports. Bugs in prefix stripping here could result in silent data corruption (e.g., merging files that shouldn't be merged or failing to match). The tests are small, directly close the mutant gaps, and run instantly.
