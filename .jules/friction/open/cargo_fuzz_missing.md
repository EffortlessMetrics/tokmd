---
id: cargo_fuzz_missing
persona: Fuzzer
style: Prover
shard: interfaces
status: open
---

# Friction: Cargo Fuzz Missing

The `cargo fuzz run` command fails because `cargo-fuzz` is not installed by default in the execution environment. The Fuzzer persona requires nightly Rust and `cargo-fuzz` to execute the defined fuzz targets in the `fuzz/` directory.

To resolve this, `cargo-fuzz` must be explicitly installed via `cargo +nightly install cargo-fuzz` or included in the default container image for the Fuzzer persona.
