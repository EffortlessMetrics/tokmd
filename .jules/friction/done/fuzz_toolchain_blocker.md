# Friction Item

id: fuzz_toolchain_blocker
persona: fuzzer
style: prover
shard: interfaces
status: closed

## Problem
`cargo fuzz` was not a reliable local gate in agent environments. Repeated runs
hit missing nightly-toolchain support in sandboxed Linux environments or
sanitizer/LLVM startup failures on Windows/MSVC before the target started.

## Evidence
- Initial Windows/MSVC retry built `fuzz_toml_config` but failed to start with
  `STATUS_DLL_NOT_FOUND`.
- `clang_rt.asan_dynamic-x86_64.dll` was found under Visual Studio 2022:
  `VC\Tools\MSVC\14.44.35207\bin\Hostx64\x64`.
- After prepending that directory to `PATH`, this command built and started:
  `cargo +nightly fuzz run fuzz_toml_config --features config --strip-dead-code false -- -runs=1 -max_len=1024`.
- The run completed successfully with `Done 7 runs in 0 second(s)`.

## Resolution
The fuzz runbooks now document the Windows/MSVC ASAN runtime setup and the
bounded one-run smoke command. The Fuzzer persona also requires this setup check
before treating Windows/MSVC fuzzing as blocked.

## Current fallback
If the nightly or sanitizer toolchain is unavailable in an environment, use
deterministic regression, property, or harness coverage for the same parser or
input boundary and record the tooling blocker separately.

## Done when
- [x] `cargo +nightly fuzz run <target> --features <features> -- -runs=1`
  builds and starts on Windows/MSVC after the ASAN runtime path is set.
- [x] The runbook documents the setup command sequence.
- [x] A follow-up fuzzer task can execute a target instead of falling back to
  deterministic tests on this Windows/MSVC machine.
