# Decision Record

## Option A (recommended)
Update all integration and BDD test suites that assert for `.stderr(predicate::str::is_empty().not())` when asserting a bad or misspelled subcommand to correctly verify the exact suggestion stderr message produced by `tokmd`. This directly ties down the error suggestion features tested in CLI test suites (fixing the edge-case regression / loose assertions issue for `tokmd_config::error_hints`).

- Fits repo: Yes, directly addresses "Specsmith" mission of improving regression coverage and test suite assertions, specifically closing mutation testing gaps and loose assertion gaps around the CLI interface behavior.
- Structure: Replaces vague `.is_empty().not()` assertions with strict `.contains(...)` checks matching what the `error_hints::suggestions` engine returns.
- Velocity: Prevents future regressions in CLI parser hints.
- Governance: Improves strictness.

## Option B
Modify CLI test suites to test all combinations of flag configurations for unknown subcommands.
- When to choose: If the current test suites lack sufficient coverage over CLI error output shapes (but they already do, they just lack precise assertions).
- Trade-offs: Bloats the test suite without actually tightening the assertions on the specific `error_hints` output.

## Decision
Choosing Option A because it specifically targets the vague test assertions in the `interfaces` shard (CLI testing suites) and tightens them to verify the concrete suggestion string returned by `tokmd-config/src/error_hints.rs`.
