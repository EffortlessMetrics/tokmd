# Decision

## Option A (Deterministic proptest regressions for scan_args)
- Extract the same deterministic invariant checks from `fuzz/fuzz_targets/fuzz_scan_args.rs` and port them to a `proptest` suite in `crates/tokmd/tests/properties.rs`.
- **Trade-offs**: Fast, runs deterministically in CI, doesn't require nightly rustc or C sanitizers. Directly satisfies the "deterministic regression or harness commands" fallback gate expectation since the `fuzz_scan_args` fuzz target fails to build locally due to `libfuzzer-sys` link errors.

## Option B (Debug libfuzzer-sys link failure)
- Debug why `cargo fuzz` compilation of `fuzz_scan_args` fails with `undefined symbol: __sancov_gen_`.
- **Trade-offs**: High risk of rabbit-holing into target-specific C/C++ linker flags, LLVM sanitizer flags, and nightly rustc bugs. Violates the instruction to not wait or get blocked.

## Decision: Option A
We will implement Option A because it locks in real edge cases for parser surfaces while avoiding environmental blockers.
