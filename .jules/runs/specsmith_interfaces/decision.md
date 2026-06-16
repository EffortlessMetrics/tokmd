## 🧭 Options considered

### Option A (recommended)
Fix bug in `check_ignore` where it printed the wrong error message format for non-existent files.
- what it is: Update `crates/tokmd/src/commands/check_ignore.rs` and `crates/tokmd/src/commands/diff.rs` to correctly output "Path not found: ..." for missing paths. Also update `crates/tokmd-scan/src/tokeignore/mod.rs` to be aligned.
- why it fits this repo and shard: It aligns error messages to the expected format so that `error_hints` correctly triggers. This fits the `interfaces` shard and the `Specsmith` persona's objective to fix edge case regressions and improve polish around CLI interfaces.
- trade-offs: Structure / Velocity / Governance: Low risk change that improves the CLI error handling.

### Option B
Add robust integration tests verifying error handling across all subcommands.
- what it is: Add a comprehensive test suite in `tests/` verifying all commands output the correct "Path not found" format.
- when to choose it instead: If the problem isn't fixed yet and tests are lacking.
- trade-offs: Doesn't actually fix the bug in `check-ignore`, `diff`, or `init`.

## ✅ Decision
Option A. I have fixed the bug across `check-ignore`, `diff`, and `init` commands and verified it against existing test suite which now passes successfully.
