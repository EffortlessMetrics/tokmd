# Decision

## Option A
Strengthen coverage in `crates/tokmd-types` by ensuring tests catch mutants related to `TokenEstimationMeta`, `TokenAudit` struct factory methods, and `is_default_policy()`. Specifically:
- `ToolInfo::current`
- `TokenEstimationMeta::from_bytes` and `from_bytes_with_bounds`
- `TokenAudit::from_output` and `from_output_with_divisors`

These structures are exposed APIs describing token limits, audit totals, and estimations, yet several branches/mutants are not covered by the current tests or only incidentally.

By introducing explicit, boundary-focused checks (like testing zero values, verifying the actual maths involved, and confirming that the default tool version is distinct from the current one), we can close the mutant gaps in `tokmd-types` permanently and effectively.

Trade-offs: High confidence, relatively small scope, low risk. Directly satisfies the mission of the `Mutant` persona inside `core-pipeline`.

## Option B
Add mutation coverage for `crates/tokmd-format` where a number of rendering mutants might be un-covered.

Trade-offs: `cargo mutants` on `tokmd-format` takes over 5 minutes (due to heavy macro or insta usage), causing timeouts in our environment and making it difficult to pinpoint exactly which mutants to tackle without running it piecemeal. Thus, `tokmd-types` is a more practical and reliable target.

## Decision
**Option A**. It's within the shard (`core-pipeline`), explicitly resolves gaps identified by `cargo mutants`, and we already have a passing test written (`mutant_coverage.rs`) that catches many of them.
