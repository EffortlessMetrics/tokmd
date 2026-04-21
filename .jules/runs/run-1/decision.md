## Decision

**Option A (Property testing for new interfaces)**
- What it is: Adding `proptest` round-trip property tests for `SensorFormat`, `NearDupScope`, and `DiffRangeMode` to `crates/tokmd-config/tests/properties.rs`.
- Why it fits: These are config enums representing input/parser surfaces, mapping exactly to Fuzzer's target "minimal harness improvements that make future fuzzing cheaper" and "deterministic regressions extracted from fuzzable surfaces". The Fuzzer tooling in the sandbox is broken due to ASAN linker errors (`__sancov_gen_.X`), so improving standard deterministic property testing is the best alternative permitted by the gate profile.
- Trade-offs: Lower velocity as it requires manual test construction, but very high structure and deterministic value without flaky fuzz timeouts.

**Option B (Skip and write a learning PR)**
- What it is: Only noting that `cargo fuzz` fails to link.
- Why it fits: We encountered tool friction.
- Trade-offs: Hallucinates away the instruction to "land deterministic regression or harness commands" if fuzz tooling is unavailable.

**Decision:** Option A
