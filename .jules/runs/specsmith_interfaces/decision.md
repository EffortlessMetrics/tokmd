# Decision Record

## What was inspected
I investigated the `crates/tokmd/tests/` folder for `tokmd`'s interfaces shard. I found a few issues with the tests for the `cockpit` command. Specifically, there were tests that were asserting `test_pct < 50.0` but some missing integration scenarios for the overall `cockpit` command using the standard BDD scenario structure (Given/When/Then). Moreover, some `tokmd` CLI commands were using tests with silent-return early-exits instead of proper panics, masking failures.

## Option A (recommended)
Add a proper BDD scenario test `given_git_repo_with_changes_when_cockpit_json_then_valid_schema` to `crates/tokmd/tests/bdd_scenarios_w75.rs` to verify that `tokmd cockpit` parses standard repository changes correctly and outputs the `CockpitReceipt` structure as a complete schema (`schema_version`, `change_surface`, `composition`). Ensure the test actually asserts the command is successful rather than silently exiting.

*   Why it fits: Locks in an edge-case regression and fixes a missing BDD/integration coverage scenario for an important path (`tokmd cockpit`).
*   Trade-offs: Bounded code change directly in the assigned `interfaces` shard. High value.

## Option B
Refactor `crates/tokmd/tests/cockpit_integration.rs` to eliminate all early-return silent test closures.

*   Why it fits: Improves the robustness of the tests.
*   Trade-offs: Larger blast radius and touches existing legacy logic, deviating from adding a strong BDD scenario which is prioritized.

## Decision
Option A was chosen. I added a new BDD scenario test for the `cockpit` command in `crates/tokmd/tests/bdd_scenarios_w75.rs` to lock in its valid schema integration testing, ensuring the test panics on internal failure steps instead of returning silently.
