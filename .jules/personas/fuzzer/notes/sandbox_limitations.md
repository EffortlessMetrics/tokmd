# Fuzzer Sandbox Limitations

**Context:**
The Fuzzer persona heavily relies on `cargo fuzz` to validate and improve input hardening. However, three separate friction items (FRIC-20260413-001, FRIC-20260428-001, cargo_fuzz_asan_linker_failure) have consistently demonstrated that `cargo fuzz` does not function out-of-the-box in the standard Jules execution environment.

**Observed Limitations:**
1. **Missing Toolchain & Tools:** `cargo-fuzz` and the `nightly` toolchain are not installed by default in the sandbox (`rustup default nightly` required).
2. **ASAN Linker Failures:** Even if installed, compiling fuzzer targets fails during the linking stage with undefined reference errors pointing to ASAN (e.g., `rust-lld: error: undefined symbol: __sancov_gen_.279`), likely due to mismatched llvm toolchains or missing ASAN-compatible C compiler.
3. **Windows/MSVC Fails:** `cargo +nightly fuzz run` still fails on Windows/MSVC targets (`_RNvXsU_... does not exist`) before the target can run.

**Guidance for Future Runs:**
If fuzz tooling fails due to these sandbox limitations, do not repeatedly log friction or force pseudo-fuzz claims. Instead, gracefully fallback to the `fuzz` gate profile expectation: write deterministic regression tests or harness commands that prove the issue without relying on ASAN fuzzing execution.