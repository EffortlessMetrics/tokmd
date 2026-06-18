# Decision

## Option A (recommended)
Add `proptest` coverage to `crates/tokmd-settings/tests/properties.rs` that explicitly validates `TomlConfig::parse` behaves deterministically and never panics on arbitrary string inputs `\\PC*` (which covers edge-cases that fuzzing typically uncovers). This acts as a deterministic regression test directly addressing config parser robustness.

- Fits the repo and shard by adding deterministic parser robustness proofs where `cargo fuzz` locally fails due to ASAN / missing nightly limits.
- Validates the `tokmd-settings` parser surface without requiring complex infrastructure.
- Follows the instructions regarding missing fuzz infrastructure by falling back to deterministic regressions on fuzzable surfaces.

## Option B
Attempt to use `cargo fuzz` directly despite the known ASAN linker limitations in the environment.

- Likely to fail with `-Zsanitizer=address` toolchain errors or undefined references.
- Slower execution, breaks the requirement of prompt completion.
- Trade-off: Lower velocity and higher risk of timeout/failure in standard environment.

## Decision
Choose Option A. Adding `toml_config_parse_no_panic` as a property test provides immediate deterministic proof that the parser surface does not crash on malformed input, satisfying the Fuzzer persona goals while avoiding known tooling blocks.
