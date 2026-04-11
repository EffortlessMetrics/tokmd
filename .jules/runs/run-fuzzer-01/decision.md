## Option A (Recommended)
Update the `fuzz_toml_config` fuzz target and the corresponding `config` feature in `fuzz/Cargo.toml` to depend on `tokmd_settings` instead of `tokmd_config`. This fixes the build breakage for this target caused by the crate refactor, and keeps the fuzzing suite compiling cleanly.

## Option B
Delete the fuzz target.

## Decision
Option A. Deleting the target loses test coverage for an important component (parser/input). Option A aligns exactly with the memory hint and directly resolves the issue in `interfaces`.
