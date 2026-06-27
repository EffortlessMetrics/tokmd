# Friction Item

id: cargo_fuzz_asan_linker_failure
persona: fuzzer
style: prover
shard: interfaces
status: done
superseded_by: fuzz_toolchain_blocker

## Problem
Running `cargo fuzz run <target> --features <features> -- -runs=N` on any fuzzer target (like `fuzz_scan_args` or `fuzz_toml_config`) fails during the linking stage with undefined reference errors pointing to ASAN:
`rust-lld: error: undefined symbol: __sancov_gen_.279`

## Evidence
- The execution environment natively lacks `cargo-fuzz`. Installing `cargo-fuzz` and switching to `nightly` via `rustup` successfully downloads the tools, but compiling fails because the sanitizer coverage (`-Zsanitizer=address`) cannot find its own linker symbols, likely due to mismatched llvm toolchains or a missing ASAN-compatible C compiler.

## Why it matters
The user specifically requested fuzz-related work. In the absence of viable fuzz tooling due to execution environment friction, we must fallback to the `fuzz` gate profile expectation.

## Done when
- [x] Documented this blocker in friction item.
