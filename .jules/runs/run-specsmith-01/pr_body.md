## 💡 Summary
Added explicit integration test coverage to lock in the exact CLI error output for unrecognized subcommands. This ensures the helpful fallback hints correctly distinguish between typos and nonexistent paths.

## 🎯 Why
Missing BDD/integration coverage for an important edge-case error path. The existing error handling tests merely checked that `stderr` was not empty when providing an unknown subcommand. However, `tokmd`'s CLI logic treats unmatched positional arguments as paths for the default `lang` subcommand, producing a custom path-not-found error message that contains important hints (e.g., "If this was meant to be a subcommand, it is not recognized"). This custom error hint is a critical piece of the CLI's UX and needs to be locked down so future `clap` or CLI parser updates do not accidentally revert it to a confusing `clap` parse error or a generic file-not-found error.

## 🔎 Evidence
- `crates/tokmd/tests/cli_e2e_w58.rs` and `error_handling_w70.rs` only assert `!stderr.is_empty()`.
- Running `cargo run -- frobnicate` produces:
```
Error: Path not found: frobnicate

Hints:
- Verify the input path exists and is readable.
- Use an absolute path to avoid working-directory confusion.
- If this was meant to be a subcommand, it is not recognized. Use `tokmd --help`.
```

## 🧭 Options considered
### Option A (recommended)
- What it is: Write a dedicated integration test to explicitly assert the exact text of the fallback error message.
- Why it fits this repo and shard: It directly targets the `interfaces` shard and satisfies the Specsmith persona's goal to improve edge-case regression coverage and lock in behavior with behavior-level tests.
- Trade-offs:
  - Structure: Minimal impact, cleanly encapsulated in a new integration test file.
  - Velocity: Extremely fast and low risk.
  - Governance: Follows standard rust integration testing.

### Option B
- What it is: Try to fix `clap` config such that it properly rejects subcommands before evaluating positional path arguments.
- When to choose it instead: If the current behavior was deemed a bug.
- Trade-offs: This would break the ergonomic default `lang` subcommand behavior (e.g. `tokmd src/` -> `tokmd lang src/`).

## ✅ Decision
Option A. The current behavior is intentional and ergonomic. We need to lock in the exact custom error text that explains this fallback behavior to users so it does not regress.

## 🧱 Changes made (SRP)
- `crates/tokmd/tests/cli_error_unknown_subcommand_specsmith.rs`: Created new integration test.

## 🧪 Verification receipts
```text
$ cargo run -- frobnicate 2> stderr.txt
Error: Path not found: frobnicate

Hints:
- Verify the input path exists and is readable.
- Use an absolute path to avoid working-directory confusion.
- If this was meant to be a subcommand, it is not recognized. Use `tokmd --help`.

$ cargo test -p tokmd --test cli_error_unknown_subcommand_specsmith
running 1 test
test test_unknown_subcommand_exact_error_message ... ok
```

## 🧭 Telemetry
- Change shape: New test file.
- Blast radius: Tests only. Zero runtime impact.
- Risk class: Low risk. Test addition only.
- Rollback: Revert the PR.
- Gates run: `cargo test -p tokmd --test cli_error_unknown_subcommand_specsmith`

## 🗂️ .jules artifacts
- `.jules/runs/run-specsmith-01/envelope.json`
- `.jules/runs/run-specsmith-01/decision.md`
- `.jules/runs/run-specsmith-01/receipts.jsonl`
- `.jules/runs/run-specsmith-01/result.json`
- `.jules/runs/run-specsmith-01/pr_body.md`

## 🔜 Follow-ups
None
