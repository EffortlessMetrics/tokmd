## Option A (recommended)
- what it is: Follow the Fuzzer persona constraints regarding environmental blocks, acknowledge the ASAN/nightly failure of `cargo fuzz`, and create a Learning PR with a friction item instead of hallucinating work outside of parser/input fuzzability.
- why it fits this repo and shard: Memory states that when operating under the `fuzz` gate profile and `cargo fuzz` fails due to ASAN linking errors in the sandbox, we must not attempt to fix the sandbox toolchain, but rather pivot to providing deterministic regression tests as required by the fallback gate expectations. However, since the instruction to "not stop without a final `result.json` and `pr_body.md`" and the fallback explicitly mentions "if no honest code/docs/test patch is justified, finish with a learning PR", creating a learning PR about this block is completely compliant. On the other hand, the memory also states: "In the `tokmd` sandbox environment, `cargo +nightly fuzz run` may fail due to ASAN (AddressSanitizer) linking errors. Do not attempt to fix the sandbox toolchain; instead, pivot to providing deterministic regression tests as required by the fallback gate expectations." Therefore, Option A is to implement a deterministic regression or property test for config parsing (since `fuzz_toml_config.rs` is in the `crates/tokmd-config/**` path equivalent, `tokmd-settings`).
- trade-offs: Option A requires writing property/regression tests to mimic fuzzing, which provides immediate deterministic value but less exploratory depth than true fuzzing.

## Option B
- what it is: Do nothing and just write a learning PR that `cargo fuzz` failed.
- when to choose it instead: If it was impossible to write deterministic tests.
- trade-offs: We would miss the opportunity to fulfill the Fuzzer persona's goal of "improving fuzzability or input hardening around parser/input surfaces" via deterministic regression or harness improvements.

Decision: I will write deterministic regression or property tests around `TomlConfig::parse` in `crates/tokmd-settings/tests/` to meet the fallback gate expectations, satisfying the Fuzzer persona's goals.
