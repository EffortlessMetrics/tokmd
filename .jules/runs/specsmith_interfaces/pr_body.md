## 💡 Summary
Strengthened CLI integration tests to explicitly assert the exact "Path not found: <cmd>" error message for unknown subcommands.

## 🎯 Why
Previously, integration tests verifying the behavior of unknown subcommands merely asserted that `stderr` was not empty. This weak assertion could pass even if the application panicked or printed an unrelated error, failing to lock in the actual intended behavior. In the `tokmd` CLI, an unknown subcommand is intentionally parsed as a positional path argument for the default `lang` subcommand, resulting in a specific "Path not found" error. This patch enforces that specific contract.

## 🔎 Evidence
- `crates/tokmd/tests/error_handling_w70.rs` and other integration test files contained weak assertions like `.stderr(predicate::str::is_empty().not());`.
- Running `cargo run -p tokmd -- nonexistent_subcommand_w70` produces: `Error: Path not found: nonexistent_subcommand_w70`.

## 🧭 Options considered
### Option A (recommended)
- Update the existing integration tests to assert the exact `Path not found: <cmd>` error string instead of a generic non-empty check.
- This locks in the documented, intentional behavior of `clap` treating unknown commands as path arguments to the default `lang` subcommand without changing the runtime.
- Trade-offs: High velocity and safety, firmly locks in existing contract.

### Option B
- Change the parser definition in `tokmd-config` to throw a standard `unrecognized subcommand` error.
- Choosing this would break the intentional shorthand of allowing `tokmd <path>` to default to `tokmd lang <path>`.

## ✅ Decision
Chose Option A. It locks in the correct, intentional behavior and prevents future regressions without altering the runtime parser logic.

## 🧱 Changes made (SRP)
- `crates/tokmd/tests/cli_e2e_w58.rs`
- `crates/tokmd/tests/cli_e2e_w69.rs`
- `crates/tokmd/tests/cli_errors_w66.rs`
- `crates/tokmd/tests/error_handling.rs`
- `crates/tokmd/tests/error_handling_w70.rs`

## 🧪 Verification receipts
```text
$ cargo run -p tokmd -- nonexistent_subcommand_w70
Error: Path not found: nonexistent_subcommand_w70

Hints:
- Verify the input path exists and is readable.
- Use an absolute path to avoid working-directory confusion.
- If this was meant to be a subcommand, it is not recognized. Use `tokmd --help`.

$ cargo test -p tokmd
...
test result: ok. 53 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s
```

## 🧭 Telemetry
- Change shape: Test modification only.
- Blast radius: Zero API/IO/docs impact. Isolated to `tokmd/tests/`.
- Risk class: Low. Test-only change.
- Rollback: Revert the assertions back to `is_empty().not()`.
- Gates run: `cargo test -p tokmd`.

## 🗂️ .jules artifacts
- `.jules/runs/specsmith_interfaces/envelope.json`
- `.jules/runs/specsmith_interfaces/decision.md`
- `.jules/runs/specsmith_interfaces/receipts.jsonl`
- `.jules/runs/specsmith_interfaces/result.json`
- `.jules/runs/specsmith_interfaces/pr_body.md`

## 🔜 Follow-ups
None.
