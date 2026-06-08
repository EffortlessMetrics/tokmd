# Cargo fuzz is not installed in the Jules environment
During `fuzzer` runs where `libfuzzer` targets are added, the environment does not have `cargo-fuzz` installed. This means we cannot actually run the fuzzers (`cargo fuzz run`) to prove they work, but can only `cargo check` them.

## Impact
Limits the ability to provide hard proof of fuzz target effectiveness.

## Remediation
Install `cargo-fuzz` in the sandbox environment, or update `fuzzer` expectations to explicitly require `cargo test` / `cargo check` rather than `cargo fuzz` if it's intentionally omitted.
