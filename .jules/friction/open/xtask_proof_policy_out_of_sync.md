# Friction Item: Out-of-sync proof policy test

The `proof_policy_json_reports_current_schema` test in `xtask/tests/proof_policy_w90.rs` is failing because it hardcodes the expected `scope_count` to 36. However, the `ci/proof.toml` file contains 37 scopes.

This test failure breaks the `cargo xtask gate` determinism gate expectations, but the fix involves modifying `xtask`, which is outside the `core-pipeline` shard.

**Recommendation:** Update the hardcoded `scope_count` in `xtask/tests/proof_policy_w90.rs` to 37, or implement a more dynamic way to verify the parsed scopes.
