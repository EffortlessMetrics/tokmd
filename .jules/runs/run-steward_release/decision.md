## Option A (recommended)
- **What it is**: Fix the failing test `routed_rust_small_docs_explain_result_receipt_fields` in `xtask/tests/proof_plan_w92.rs`. The test expects `docs/ci/swarm-routing.md` to explain the `router.target` and other router fields, but those explanations were removed or omitted. I'll re-add the missing field documentation and run the `xtask` test suite to ensure the fix correctly resolves the failure without causing regressions. I also noticed that `crates/tokmd-cockpit/Cargo.toml` has a hardcoded version `"1.11.0"` for `tokmd-analysis` rather than the workspace consistent `">=1.9, <2"` pattern used elsewhere.
- **Why it fits**: We're improving release/governance hygiene. The CI tests for the CI documentation are failing due to a docs drift, and the `tokmd-cockpit` manifest drift is another metadata mismatch. This addresses target #1 (publish-plan/version-consistency drift) and #3 (RC-hardening docs/checks) from the Steward prompt.
- **Trade-offs**:
    - Structure: Improves consistency and prevents future confusion.
    - Velocity: High; easy fixes for documentation and manifest.
    - Governance: Restores CI documentation truth against testing contracts.

## Option B
- **What it is**: Remove the `routed_rust_small_docs_explain_result_receipt_fields` test entirely.
- **When to choose it instead**: If the router result receipt concept was completely removed rather than just the separate routing jobs.
- **Trade-offs**: The test explicitely states that it is retained because the routed result receipt remains documented for historical run artifacts. Removing the test would cause silent drift between the historical artifact structure and the documentation, violating governance principles.

## Decision
Option A. The missing fields were simply omitted from the docs during an update, and updating the docs explicitly to mention them satisfies the test. Also fixing the `tokmd-cockpit` dependency drift resolves a version alignment issue, aligning fully with the Steward persona's goal.
