# Option A (recommended)
Add missing snapshot golden test coverage for the CLI `analyze` subcommand with the `--preset estimate` flag in `crates/tokmd/tests/cli_snapshot_golden.rs`. The output from `analyze` includes dynamic properties like `base_signature` which varies per run. Normalizing this property locking down the contract and preventing non-deterministic test failures directly satisfies the `Gatekeeper` persona's mandate to "lock in deterministic behavior" and "snapshot/golden drift or weak coverage". This provides deterministic and rigorous regression tests for contract-bearing JSON output.
* Structure: Extends existing testing suite appropriately.
* Velocity: Quick to implement, prevents future bugs.
* Governance: Aligned with deterministic goals.

# Option B
Refactor `crates/tokmd-types` or `crates/tokmd-model` internals to avoid map iterations or force absolute reproducibility of intermediate analysis structures without immediately asserting this via snapshot.
* While it might achieve determinism under the hood, we lack the outward verifiable contract (the snapshot test).
* Trade-offs: Difficult to prove correctness without an observable contract like snapshot test.

# Decision
**Option A**. The lack of `analyze` tests in the `cli_snapshot_golden.rs` file represents a clear testing gap and regression risk, particularly for the output schemas. Adding this snapshot with deterministic normalization fits perfectly under the `core-pipeline` shard and `Gatekeeper` persona constraints.
