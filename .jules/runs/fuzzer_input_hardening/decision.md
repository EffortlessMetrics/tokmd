# Decision

## What was inspected
- `crates/tokmd/tests/properties.rs`
- `crates/tokmd/tests/determinism.rs`
- `crates/tokmd/tests/determinism_regression.rs`
- `crates/tokmd/tests/determinism_hardening.rs`
- `crates/tokmd/src/cli/parser.rs`

## Options considered

### Option A (Recommended)
Add a new `fuzz_cli_w81.rs` (or similar) in `crates/tokmd/tests` that uses `proptest` to test the CLI parsing logic with arbitrary user inputs. The CLI interface is an important input surface, and adding a proptest for it improves fuzzability and guards against regressions where the CLI parser might panic on unexpected input.

*Why it fits this repo and this shard:* This fits the `fuzzer` persona well, which is tasked to "Improve fuzzability or input hardening around parser/input surfaces." Testing the CLI argument parser directly improves input hardening.

*Trade-offs:*
- Structure: Improves structural confidence in the CLI parser.
- Velocity: Adds a new test file, but it's small and quick.
- Governance: Aligns with the deterministic regression requirement.

### Option B
Modify existing determinism tests to include randomized proptest variants.

*Why it fits this repo and this shard:* Still improves testing of input surfaces.
*Trade-offs:* Modifying existing tests can be riskier and might blur the line between deterministic regression and fuzzing. Creating a dedicated CLI parsing proptest is cleaner.

## Decision
Choose Option A. I will add a new test file `crates/tokmd/tests/fuzz_cli.rs` that explicitly uses `proptest` to fuzz the `tokmd` CLI argument parser.
