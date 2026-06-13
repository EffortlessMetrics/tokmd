# Specsmith Decision

## Target Selection
We need to improve scenario coverage, regression coverage, and edge-case polish in the `interfaces` shard (Config, core facade, and CLI interfaces). The target ranking is:
1) missing BDD/integration coverage for an important path
2) edge-case regression not locked in by tests
3) confusing scenario setup that hides real behavior
4) scenario-driven sharp-edge polish

Checking the test suite, we have BDD coverage for almost all major commands (`lang`, `module`, `export`, `analyze`, `diff`, `global/config` behavior), except for two newer commands: `gate` and `cockpit`.

*   **Option A**: Add BDD scenario tests for `gate` (`bdd_gate_scenarios_w50.rs`).
    *   **Pros**: `gate` is a crucial CI policy enforcement tool. Providing a clear Given/When/Then scenario suite documents the exact behaviour of `tokmd gate` and ensures regressions are caught in user-facing workflows (e.g. ratchet failures, policy parsing).
    *   **Cons**: Relies on existing test harnesses, might overlap slightly with `gate_integration.rs` (which tests the mechanics but less the user-facing BDD contract).
*   **Option B**: Add BDD scenario tests for `cockpit` (`bdd_cockpit_scenarios_w50.rs`).
    *   **Pros**: `cockpit` is a complex command aggregating Git data and analysis. BDD tests document exactly what PR diffs produce what outputs.
    *   **Cons**: `cockpit_cli_w71.rs` already contains very deep structural checks on the output JSON/MD, acting much like integration tests. Re-writing them as BDD might be redundant or lower ROI compared to `gate` which has nuanced ratchet logic.
*   **Option C**: Fix an edge-case configuration scenario where `toml` settings interact with CLI args for commands like `sensor` or `export`. The `config_resolution.rs` is very unit-test focused.

I choose **Option A (Add BDD tests for `gate`)**.
The `tokmd gate` command represents the CI-blocking surface of the tool. User stories describing "Given a ratchet config with a 10% tolerance, When a change exceeds 10%, Then the gate fails" is the ideal candidate for BDD-style documentation tests. We have `feature_gates_w71.rs` (testing CLI flags/features) and `gate_integration.rs` (testing the mechanics), but no `bdd_gate_scenarios_w50.rs`.
