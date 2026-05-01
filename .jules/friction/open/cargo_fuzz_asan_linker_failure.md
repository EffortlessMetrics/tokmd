# Friction Item

id: FRIC-20260429-003
persona: fuzzer
style: prover
shard: interfaces
status: open

## Problem
Running `cargo fuzz run <target> --features <features> -- -runs=N` on any fuzzer target fails during the linking stage with undefined reference errors pointing to ASAN (`rust-lld: error: undefined symbol: __sancov_gen_.279`).

## Evidence
- commands: `cargo fuzz run fuzz_scan_args --features analysis -- -runs=1`
- outputs: `rust-lld: error: undefined symbol: __sancov_gen_.279`

## Why it matters
Fuzzing fails to link in the default environment, requiring fallback to deterministic testing.

## Done when
- [ ] ASAN linker errors are resolved in the default environment
