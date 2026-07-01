# Fuzzer Decision: CLI Input Hardening

## Background
The Fuzzer persona is tasked with improving input hardening or fuzzability, particularly around parser/config/input surfaces. I surveyed the `interfaces` shard (crates/tokmd-config, crates/tokmd-core, crates/tokmd).

Fuzzing tooling (`cargo fuzz`) is not available in the environment, so the gate profile and runbook dictate providing deterministic regressions extracted from fuzzable surfaces.

I found a property test suite for the CLI parser (`crates/tokmd/tests/cli_parser_properties.rs`). This suite explicitly states that the `tokmd::cli::Cli` parser should *never panic when fed arbitrary string arguments*.

I created a deterministic regression test (`crates/tokmd/tests/cli_parser_fuzz_regression.rs`) mimicking a fuzzing mutation: feeding invalid UTF-8 bytes to an argument expecting valid strings (e.g. `--exclude`). The test verifies that `clap` handles this safely by rejecting it with an `ErrorKind::InvalidUtf8` rather than crashing/panicking. This directly serves the `fuzz` gate profile and Fuzzer persona's goal of locking in deterministic proof of parser input hardening.

## Option A (Recommended)
Land the deterministic regression test for the CLI parser demonstrating safe rejection of invalid UTF-8 byte arrays, expanding the property test suite's coverage of input hardening.

*   **Structure:** Minimal disruption. Fits neatly alongside existing property tests.
*   **Velocity:** Fast. The test works, proves safe rejection of fuzzed/malformed inputs, and does not require changes to core code (since clap handles this).
*   **Governance:** Perfectly aligns with the `fuzz` profile fallback expectations.

## Option B
Attempt to implement manual fuzzer stubs (e.g., AFL, libfuzzer-sys manually).

*   **Trade-offs:** High friction, violates instructions (cargo-fuzz is unsupported, do not fix it), high chance of build errors.

## Decision
Option A. It's safe, proven, and explicitly meets the Fuzzer requirements to provide deterministic input hardening proof when fuzzers are unavailable.
