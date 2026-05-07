# Friction Item: Brittle assertion in proof policy tests

The test `proof_policy_json_reports_current_schema` inside `xtask/tests/proof_policy_w90.rs` initially hardcoded an exact assertion for the number of scopes in `ci/proof.toml`. When policies are added or removed, this test breaks unnecessarily, causing factual drift that requires manual patching. This was superseded by an architectural fix (#1722) which removed the brittle exact assertion.

This item records the workflow collision and learning.
