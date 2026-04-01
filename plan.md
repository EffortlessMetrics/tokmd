1. **Discover & Select Target**:
   - Lane B (Scout discovery): Target feature-flag interaction that breaks tests.
   - Specifically, test files heavily testing the `handoff` subcommand or `git` feature must be conditionally compiled with `#![cfg(feature = "git")]` at the top.
   - Files targeted:
     - `crates/tokmd/tests/handoff_w71.rs`
     - `crates/tokmd/tests/handoff_integration.rs`
     - `crates/tokmd/tests/context_cli_w73.rs`
     - `crates/tokmd/tests/deep_context_handoff_w51.rs`
     - `crates/tokmd/tests/deep_cli_complex_w48.rs`
     - `crates/tokmd/tests/context_handoff_deep.rs`
     - `crates/tokmd/tests/docs.rs`

2. **Implement**:
   - Use `replace_with_git_merge_diff` or bash `sed` to insert `#![cfg(feature = "git")]` at the beginning of each file.

3. **Verify**:
   - Run `cargo test --no-default-features` and `cargo test --all-features` to ensure no test or compilation failures.
   - Record the receipts.

4. **Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.**
   - Run `pre_commit_instructions` tool.

5. **Submit PR**:
   - Title: `compat: fix no-default-features testing for handoff 🧷 Compat`
   - Body: Follow PR Glass Cockpit template, noting determinism/telemetry. Include receipts.
