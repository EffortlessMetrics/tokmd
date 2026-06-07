# Friction Item: cargo-fuzz requires nightly toolchain in sandbox

## Context
Attempted to run `cargo fuzz` targets locally as the `Fuzzer` persona. The standard `cargo fuzz` workflow relies on `libfuzzer-sys` and `-Zsanitizer=address`, which are only available on the nightly compiler.

## Impact
The default agent environment does not have a working `cargo fuzz` setup because it defaults to stable. This blocks the primary fuzzer workflow, forcing a fallback to deterministic tests, which may already be saturated (like the `cli_parser_properties.rs` tests).

## Suggestion
Evaluate updating the sandbox image to include rustup nightly for fuzzing tasks, or establish a convention for running stable-compatible fuzzing alternatives without ASAN.
