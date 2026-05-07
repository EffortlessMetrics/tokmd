# Investigation

Looked at test outputs and specifically saw a failure when checking deterministic properties of `proof.toml` policy schema check:
`thread 'proof_policy_json_reports_current_schema' panicked at xtask/tests/proof_policy_w90.rs:191:5: assertion left == right failed` where `left: Number(40)` but right was 38.

This means that while the `ci/proof.toml` had 40 scopes, the test `proof_policy_json_reports_current_schema` was incorrectly hardcoded to expect 38, causing a regression test failure.
There were also `cargo xtask check-no-panic-family` and other checks run to ensure determinism properties remain true.

# Options considered

### Option A (recommended)
Update the test assertion in `xtask/tests/proof_policy_w90.rs` to correctly match the schema drift and pass the deterministic test for `proof.toml`.

- fits this repo and shard: This test specifically verifies the structure and correctness of the tooling's output for proof policy.
- trade-offs:
    - Structure: Fixes factual test assertion mismatch without changing runtime logic.
    - Velocity: Very fast fix.
    - Governance: Restores green tests for deterministic schema validation.

### Option B
Ignore it or delete the assertion.

- trade-offs: This weakens the testing surface, increasing the risk of future untested drift.

# Decision
Option A. Updated the test to expect the correct scope count (40) from the updated `ci/proof.toml`.
