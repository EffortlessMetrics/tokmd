# Decision

## Option A (Add missing BDD test for `--children parents-only` in `module` command)
Add a missing BDD integration test in `crates/tokmd/tests/bdd_module_scenarios_w50.rs` for the `module` command using `--children parents-only`.
The `--children` argument accepts different values depending on the subcommand: `tokmd module` accepts `separate` or `parents-only`, while `tokmd lang` accepts `separate` or `collapse`.
While there is a test for `--children separate` for the `module` subcommand (`given_project_when_module_children_separate_then_mode_recorded`), there is no test verifying that `parents-only` is successfully passed to the engine and recorded in the JSON output arguments.

**Trade-offs:**
- **Structure:** Improves test coverage for CLI argument handling and JSON output.
- **Velocity:** Small, fast addition that perfectly locks in existing behavior.
- **Governance:** Keeps scenario tests aligned with the actual capabilities of the CLI and its schema.

## Option B (Add learning PR)
Create a learning PR because no suitable scenario tests were found to add.

**Trade-offs:**
- Misses an opportunity to improve BDD test coverage for an actual capability of the CLI that lacked a dedicated scenario check.

## Decision
**Option A.** It clearly fits the Specsmith persona to close scenario gaps ("missing BDD/integration coverage for an important path" / "edge-case regression not locked in by tests"), and specifically addresses the unique `parents-only` vs `collapse` argument behavior between subcommands.
