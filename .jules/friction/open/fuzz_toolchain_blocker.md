id: fuzz_toolchain_blocker
persona: fuzzer
style: prover
shard: interfaces
status: open

## Problem
`cargo fuzz` is not a reliable local gate in the current agent environments. Repeated runs hit either missing nightly-toolchain support in sandboxed Linux environments or sanitizer/LLVM link failures on Windows/MSVC before the target starts.

## Evidence
- Linux sandbox: `cargo fuzz run fuzz_policy_toml` failed because `-Zsanitizer=address` requires nightly.
- Windows/MSVC: `cargo +nightly fuzz run fuzz_toml_config --features config -- -max_total_time=1` failed with a rustc/LLVM associative COMDAT sanitizer-link error.
- Older Windows attempts also failed during ASAN sanitizer coverage linking with unresolved sanitizer symbols.

## Current workaround
When the fuzzer gate is blocked by toolchain infrastructure, land deterministic regression/proptest/check coverage for the same parser or input boundary, and record the fuzz tooling blocker separately.

## Done when
- [ ] `cargo +nightly fuzz run <target> --features <features> -- -runs=1` builds and starts on Windows/MSVC.
- [ ] The default sandbox image either provides nightly fuzz tooling or the runbook documents the setup command sequence.
- [ ] A follow-up fuzzer task can execute the target instead of falling back to deterministic tests.
