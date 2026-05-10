## Options Considered

### Option A: Create a proptest-based parser fuzzer for the CLI in `crates/tokmd/tests/` (Recommended)
- **What it is:** A property-based test module in `crates/tokmd/tests/cli_parser_properties.rs` using the `proptest` crate to fuzz `clap::Parser::try_parse_from`. It feeds arbitrary lists of string arguments to the tokmd CLI parser to prove that parsing malformed inputs never causes panics.
- **Why it fits this repo and shard:** The CLI parser (`tokmd::cli::Cli`) is a primary input surface in the `interfaces` shard. While `libfuzzer-sys` is tricky to compile locally without `nightly`, `proptest` is already heavily used in the `tokmd` codebase (`tests/properties.rs`, `tests/proptest_expansion_w50.rs`) for deterministic regressions and fuzzable surface coverage. This acts as a robust, deterministic proxy for input hardening that works on stable Rust.
- **Trade-offs:**
  - *Structure:* Aligns perfectly with the existing `Prover` test philosophy in `tokmd`.
  - *Velocity:* High, can be merged cleanly and runs automatically in CI.
  - *Governance:* Safer than true fuzzers since it's fully deterministic and doesn't require nightly compiler infrastructure for regular developers.

### Option B: Fix the `cargo-fuzz` setup
- **What it is:** Attempt to fix the linker issues (`__sancov_gen_` undefined symbols) when running `cargo fuzz` locally.
- **When to choose it instead:** When the primary goal is deep mutation testing on byte-streams rather than structured argument parsing.
- **Trade-offs:**
  - The repo specifies "deterministic regressions extracted from fuzzable surfaces" or deterministic proptests as valid alternatives if fuzz tooling is unavailable. `cargo-fuzz` frequently breaks due to nightly compiler churn and linker issues. Time spent fixing environmental issues is better spent on concrete input coverage.

## Decision
**Option A**. Proptest-based fuzzing of the CLI parser directly proves the invariant (the CLI should never panic on arbitrary input) and runs deterministically across all environments without nightly Rust requirements. It's a high-value proof-improvement patch.
