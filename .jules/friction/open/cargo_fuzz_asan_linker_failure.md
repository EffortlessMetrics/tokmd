# Friction: cargo fuzz fails to build fuzz targets due to ASAN linker errors

**Symptom**: Running `cargo fuzz run <target> --features <features> -- -runs=N` on any fuzzer target (like `fuzz_scan_args` or `fuzz_toml_config`) fails during the linking stage with undefined reference errors pointing to ASAN:
`rust-lld: error: undefined symbol: __sancov_gen_.279`

**Context**: The execution environment natively lacks `cargo-fuzz`. Installing `cargo-fuzz` and switching to `nightly` via `rustup` successfully downloads the tools, but compiling fails because the sanitizer coverage (`-Zsanitizer=address`) cannot find its own linker symbols, likely due to mismatched llvm toolchains or a missing ASAN-compatible C compiler.

**Workaround**: The user specifically requested fuzz-related work. In the absence of viable fuzz tooling due to execution environment friction, we must fallback to the `fuzz` gate profile expectation: "otherwise deterministic regression or harness commands". So I've chosen to fix an unhandled parser/input problem identified in the `tokmd` CLI inputs where subcommands were swallowed as positional missing paths. However, since the prompt specifies this is a "learning PR", we should revert the feature fix and focus purely on documenting this blocker.
