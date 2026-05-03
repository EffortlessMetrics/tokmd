# Friction Item: Fuzz Toolchain Blocker

id: FRIC-fuzz-toolchain-blocker
persona: fuzzer
style: prover
shard: interfaces
status: open

## Problem
`cargo fuzz` fails to run in the default Jules sandbox environment due to a combination of toolchain constraints:
1. It requires a `nightly` compiler (fails out-of-the-box with "the option `Z` is only accepted on the nightly compiler").
2. When `nightly` is installed, linking fails with undefined reference errors pointing to ASAN (`rust-lld: error: undefined symbol: __sancov_gen_.279`).
3. On Windows/MSVC, it fails with missing COMDAT symbols (`rustc-LLVM ERROR: Associative COMDAT symbol...`).

## Evidence
- files / paths: `fuzz/fuzz_targets/*`
- commands: `cargo fuzz run fuzz_policy_toml` or `cargo +nightly fuzz run fuzz_toml_config --features config -- -max_total_time=1`
- related run ids: `FRIC-20260413-001`, `FRIC-20260428-001`, `cargo_fuzz_asan_linker_failure`

## Why it matters
The Fuzzer persona heavily relies on `cargo fuzz` to validate and improve input hardening. Without nightly toolchain support out-of-the-box in the sandbox and properly configured ASAN linkers, fuzzing tasks are blocked as infrastructure friction. This forces fuzz runs to gracefully fallback to deterministic testing or documentation.

## Done when
- [ ] Sandbox image provides a nightly toolchain by default.
- [ ] ASAN linker errors are resolved in the default environment.
- [ ] Or `cargo fuzz` gracefully falls back without ASAN.
- [ ] The Windows/MSVC LLVM/COMDAT issue no longer reproduces.
