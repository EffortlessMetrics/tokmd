# Option A: Add snapshot tests for `tokmd analyze --preset` commands

* **What it is:** Update `crates/tokmd/tests/cli_snapshot_golden.rs` to include deterministic snapshot tests for the `tokmd analyze` command and its presets (like `receipt`, `estimate`, `health`). Normalization will be expanded to properly scrub paths and timestamps in `effort` metrics and derivations.
* **Why it fits this repo and shard:** The shard `core-pipeline` focuses on determinism and contract-bearing schemas. `analyze` command is part of the CLI outputs which need golden snapshot tests to detect drift, directly supporting the "Gatekeeper" persona target ranking #2 (snapshot/golden drift or weak coverage).

# Option B: Add exhaustive property testing to `tokmd-model` for analysis metrics

* **What it is:** Add property tests using `proptest` verifying invariants for specific effort calculations within `tokmd-analysis-effort` (e.g. Cocomo metrics).
* **Why to choose it instead:** Directly tests properties rather than snapshots.

# Decision

**Option A**. The assignment notes "Target ranking: 2) snapshot/golden drift or weak coverage" and "Prefer tightening deterministic output, snapshot, and golden coverage before changing behavior". The CLI snapshot tests exist (`cli_snapshot_golden.rs`), but completely omit the `analyze` subcommand despite `analyze` producing large, complex json output that is ripe for regression. Option A provides the most robust regression test coverage (snapshot) for this contract-bearing output.
