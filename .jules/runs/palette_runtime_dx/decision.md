# Decision

## Option A (recommended)
Fix `tokmd check-ignore <nonexistent-path>` to report "Error: Path not found: <nonexistent-path>" instead of "Error: Path '<nonexistent-path>' does not exist". This aligns the error wording for `check-ignore` with all the other subcommands which report `Error: Path not found:` when a file doesn't exist. This also enables our general hint generator for "Path not found" to suggest to verify if the path exists, rather than just returning a plain error.

- **Structure**: This requires updating `crates/tokmd/src/commands/check_ignore.rs` and the tests that verify the exact error text.
- **Velocity**: This is a simple fix.
- **Governance**: Low risk, improves CLI UX consistency.

## Option B
Do not fix `check-ignore` error messages and find another target.

## ✅ Decision
Option A. It's a small change but directly addresses the goal of improving "unclear or low-context error messages" by aligning the error output to a standard form that benefits from existing hint logic.
