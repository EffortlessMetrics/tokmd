# Option A: Tighten snapshot coverage for `tokmd analyze` command
The `tokmd analyze` command emits a JSON receipt, but lacks golden snapshot tests verifying its entire serialized output format. While `determinism_regression` test confirms fields are stable and sorted, and `snapshot_module_json`, `snapshot_export_json`, etc. exist in `cli_snapshot_golden.rs`, `snapshot_analyze_json` is noticeably absent. Adding this snapshot completes coverage for the core pipeline's contract-bearing output formats, verifying determinism directly at the surface.
- Fits the repo and shard: It protects contract-bearing outputs and determinism without modifying core pipeline behaviors.
- Trade-offs: Increases test determinism but introduces a maintenance burden to occasionally `cargo insta accept` changes to the analysis schema.

# Option B: Add explicit missing tests for edge case flags on `tokmd analyze`
Like `tokmd module`, there might be specific flags on `analyze` not explicitly verified for determinism.
- Trade-offs: Not as comprehensively verifying the output structure as a full-snapshot test does.

# Decision
I have selected Option A. By adding `snapshot_analyze_json` in `crates/tokmd/tests/cli_snapshot_golden.rs` alongside normalization logic for fields like `base_signature` and `target_path`, the deterministic layout of the entire analysis receipt schema is now structurally locked, fulfilling the Gatekeeper persona's objective to protect deterministic output.
