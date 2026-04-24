# Decision Record

## Inspected
- `cargo-fuzz` availability and functionality in the execution environment.
- Fuzz targets inside the `fuzz/fuzz_targets/` directory, specifically `fuzz_toml_config.rs`, `fuzz_run_json.rs`, and `fuzz_scan_args.rs`.
- Target crates config surfaces: `tokmd-config`, `tokmd-core`, and `tokmd-scan-args`.

## Options considered

### Option A (recommended): Translate fuzz targets into deterministic `proptest` suites
- **What it is**: The local execution environment cannot successfully build and run `cargo fuzz` due to ASAN linker errors (`undefined symbol: __sancov_gen_...`). Rather than block on infrastructure, we can fulfill the Fuzzer persona's gate profile by porting the `libfuzzer-sys` fuzz targets into deterministic `proptest` test files in the respective crates (`tokmd-config`, `tokmd-core`, `tokmd-scan-args`).
- **Why it fits**: This satisfies the "inline fallback gate expectations" which explicitly state: "If fuzz tooling is unavailable, record N/A and land deterministic proof work instead." It improves the overall proof surfaces for parsing and input surfaces by verifying invariants against generated arbitrary inputs.
- **Trade-offs**:
  - **Structure**: Increases deterministic test coverage directly within the crate boundaries.
  - **Velocity**: Keeps the pipeline moving by converting infrastructure friction into a tangible testing win.
  - **Governance**: Fulfills the `fuzz` gate profile fallback explicitly requested in the prompt.

### Option B: Abort and record a friction item
- **What it is**: Record the `cargo-fuzz` linker failure as a blocking friction item and do not attempt to add deterministic property tests.
- **When to choose it**: If adding deterministic tests is not possible or out of scope.
- **Trade-offs**: Violates the explicit instruction to "land deterministic proof work instead" if fuzz tooling is unavailable. It creates an empty learning PR without making the codebase better.

## Decision
Choose Option A. The environment's inability to link ASAN for `cargo-fuzz` should not prevent hardening the parser surfaces. The invariant tests encoded in the fuzz targets (`fuzz_toml_config.rs`, `fuzz_run_json.rs`, and `fuzz_scan_args.rs`) are highly valuable. Porting them to `proptest` suites ensures they are run automatically during `cargo test`, increasing deterministic input hardening and satisfying the Fuzzer's mission.
