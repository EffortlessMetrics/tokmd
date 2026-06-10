# Decision

## Option A (recommended)
Add BDD tests for missing edge cases around path resolution (e.g., when the target path doesn't exist, is empty, or is a long string) to `crates/tokmd/tests/edge_cases_cli_w50.rs` or `crates/tokmd/tests/cli_errors_w66.rs`. It ensures robustness of edge cases but the test already exist in those files.

## Option B
Examine config parsing in `crates/tokmd/src/config.rs` and verify edge cases where `profile` from CLI might conflict or overwrite TOML settings, or ensure config edge cases are adequately proven.
In `config.rs`, the logic `sanitized_selector` and `get_profile_name` handles some edge cases but we can ensure BDD coverage for edge cases like when `tokmd.toml` and `config.json` have conflicting files.

## Option C
Examine JSON configuration fallback to TOML and make sure we have robust edge-case coverage for precedence and missing values in `crates/tokmd/tests/config_resolution.rs`.

Let's look at `crates/tokmd/tests/config_resolution.rs`.
