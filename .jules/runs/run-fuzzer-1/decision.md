# Decision

## 🧭 Options considered

### Option A: Improve `fuzz_toml_config` corpus (Recommended)
- **What it is**: The `fuzz_toml_config` target lacks a seed corpus entirely. This leads to inefficient fuzzing since it has to learn TOML syntax from scratch. We can add some valid and invalid `tokmd.toml` configuration files as seeds to bootstrap the fuzzer.
- **Why it fits**: The prompt asks to "Focus on parser/config/input fuzzability, corpora, or deterministic regressions extracted from fuzzable surfaces." Adding a seed corpus directly addresses this.
- **Trade-offs**:
  - *Structure*: Adds new files to the repo, but strictly within the established `fuzz/corpus/` directory layout.
  - *Velocity*: Highly effective at finding real bugs faster by giving the fuzzer valid structure to mutate.
  - *Governance*: Complies with the `fuzz` gate profile and the Fuzzer persona goals.

### Option B: Add deterministic tests for `TomlConfig` edge cases
- **What it is**: Create a new test file in `crates/tokmd-settings/src/config.rs` or a separate integration test to parse tricky TOML configurations deterministically.
- **Why it fits**: Acts as a fallback for the fuzz profile ("deterministic regression or harness commands").
- **Trade-offs**: Does not directly improve the fuzzer's efficiency as much as a corpus does, but guarantees execution in CI without `cargo-fuzz`.

## ✅ Decision
I will pursue **Option A** (improve `fuzz_toml_config` corpus) along with **Option B** (add deterministic tests) as a fallback since the fuzzer couldn't run properly in the environment (timed out during a basic `cargo run --bin fuzz_toml_config`). This satisfies both the primary instruction to improve corpus and the fallback instruction to provide deterministic coverage.
