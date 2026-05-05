# Decision

## Option A
Since the standard `cargo fuzz` toolchain requires nightly features that are not natively available or fail locally in this sandbox (due to missing nightly compiler options in the default CI/execution environment), I will extract the core logic from one of the un-runnable fuzz targets (`fuzz_toml_config.rs` or `fuzz_json_types.rs`) and convert it into a set of deterministic property tests using `proptest`. The `fuzz_toml_config.rs` target validates critical invariants on TOML configuration parsing that we can test via property testing instead.
- **Why it fits:** It honors the primary objective (improving proof surfaces in interfaces/parser/config) without relying on broken tools. We can port `fuzz_toml_config` logic to a `config_proptests.rs` file within `crates/tokmd/tests/` (which is within the allowed paths).

## Option B
Attempt to install nightly Rust to run `cargo fuzz`. This introduces environment-mutating friction, and might not succeed depending on the environment's network/permissions.

## Decision
I will proceed with **Option A**, extracting the TOML fuzz target into a deterministic property test suite.
