## 💡 Summary
Tightened CLI assertion logic in integration tests to enforce strict output validation for unrecognized subcommands. Replaces vague `is_empty().not()` assertions with exact matches against `error_hints` suggestions.

## 🎯 Why
Test assertions for unknown or misspelled subcommands were previously relying on vague `.stderr(predicate::str::is_empty().not())` checks. This failed to lock in the actual CLI output behavior (like "Did you mean the subcommand X?" or "If Y was intended as a subcommand..."). Updating these test assertions locks in the real edge-case behavior and prevents silent regressions in `tokmd`'s CLI parsing feedback.

## 🔎 Evidence
- `crates/tokmd/tests/cli_e2e_w58.rs`, `crates/tokmd/tests/cli_e2e_w65.rs`, and others.
- Observed that running an invalid subcommand like `frobnicate` outputs "If `frobnicate` was intended as a subcommand...", but tests were only checking that stderr was not empty.
- Receipt: `cargo run -p tokmd -- frobnicate` -> `Error: Path not found: frobnicate \n\nHints: ... If \`frobnicate\` was intended as a subcommand, it is not recognized.`

## 🧭 Options considered
### Option A (recommended)
- what it is: Update `.is_empty().not()` assertions to strict `.contains(...)` checks matching what the `error_hints::suggestions` engine returns.
- why it fits this repo and shard: Directly targets the Specsmith mission of improving test assertions and closing gaps around interface behavior (specifically `error_hints` logic).
- trade-offs:
  - Structure: Improves exactness and lock-in of error formats.
  - Velocity: Prevents future regressions in CLI parser hints.
  - Governance: High. Requires tests to be updated if the hint output shape changes.

### Option B
- what it is: Add a new suite of parameter exhaustive tests for bad flag and CLI values.
- when to choose it instead: If the CLI tests had poor code path coverage.
- trade-offs: Bloats the test suite without necessarily targeting the loose assertions identified in the existing test definitions.

## ✅ Decision
Chose Option A to strictly lock in the existing `error_hints` behavior without bloating the test suite with new boilerplate tests.

## 🧱 Changes made (SRP)
- `crates/tokmd/tests/cli_comprehensive.rs`: Strict assertion for `invalid_subcommand_fails`
- `crates/tokmd/tests/cli_e2e_w58.rs`: Strict assertion for `err_invalid_subcommand_shows_suggestion` and `err_completely_unknown_subcommand`
- `crates/tokmd/tests/cli_e2e_w65.rs`: Strict assertion for `err_typo_subcommand_fails` and `frobnicate_unknown_subcommand_has_stable_error_output`
- `crates/tokmd/tests/cli_e2e_w69.rs`: Strict assertion for `w69_error_unknown_subcommand`
- `crates/tokmd/tests/cli_error_paths_w51.rs`: Strict assertion for `unknown_subcommand_fails`
- `crates/tokmd/tests/cli_errors_w66.rs`: Strict assertion for `unknown_subcommand_produces_helpful_error`
- `crates/tokmd/tests/cli_pipeline_e2e_w54.rs`: Strict assertion for `invalid_subcommand_fails`
- `crates/tokmd/tests/e2e_extended.rs`: Strict assertion for `nonexistent_subcommand_fails`
- `crates/tokmd/tests/error_handling.rs`: Strict assertion for `unknown_subcommand_fails`
- `crates/tokmd/tests/error_handling_w70.rs`: Strict assertion for `unknown_subcommand_fails_with_stderr`
- `crates/tokmd/tests/regression_prevention_w55.rs`: Strict assertion for `invalid_subcommand_fails_with_help`

## 🧪 Verification receipts
```text
running 1 test
test err_invalid_subcommand_shows_suggestion ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 62 filtered out; finished in 0.01s

running 1 test
test invalid_subcommand_fails_with_help ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 44 filtered out; finished in 0.01s

cargo test -p tokmd --tests
test result: ok. 429 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s
```

## 🧭 Telemetry
- Change shape: Test modification (Proof improvement patch)
- Blast radius: `tests/` boundary only
- Risk class: Zero risk (no production code changed) + why: Tests were strengthened, not implementation.
- Rollback: `git revert`
- Gates run: `cargo clippy`, `cargo fmt`, `cargo test -p tokmd --tests`

## 🗂️ .jules artifacts
- `.jules/runs/specsmith_interfaces_01/envelope.json`
- `.jules/runs/specsmith_interfaces_01/decision.md`
- `.jules/runs/specsmith_interfaces_01/receipts.jsonl`
- `.jules/runs/specsmith_interfaces_01/result.json`
- `.jules/runs/specsmith_interfaces_01/pr_body.md`

## 🔜 Follow-ups
- N/A
