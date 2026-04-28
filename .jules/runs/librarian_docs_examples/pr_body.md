## 💡 Summary
Aligns `tokmd check-ignore` implementation with the documentation contract for missing paths. When given a path that doesn't exist, it now returns a non-zero exit code and an error message, preventing silent failures in CI.

## 🎯 Why
The CLI Reference (`docs/reference-cli.md`) explicitly notes: "As of v1.3.0, specifying a non-existent input path returns exit code 1 with an error message, rather than succeeding with empty output. This prevents silent failures in CI pipelines." However, running `tokmd check-ignore does_not_exist.rs` was silently exiting with code `0` and printing `does_not_exist.rs: not ignored (file not found)`. This drift between expected governance policy and the actual tool behavior causes confusion and breaks the documented safety guarantee for CI workflows.

## 🔎 Evidence
- `docs/reference-cli.md` states: "As of v1.3.0, specifying a non-existent input path returns exit code 1 with an error message"
- Running `tokmd check-ignore does_not_exist.rs` and `echo $?` produced exit code `0` instead of the expected failure.

## 🧭 Options considered
### Option A (recommended)
Update the `check-ignore` command to return an `Err` containing an `anyhow` error when a path doesn't exist, failing the command execution.
- why it fits this repo and shard: It enforces deterministic behavior in CI environments and directly aligns with the `tooling-governance` shard's goal of preventing factual drift between behavior and docs.
- trade-offs: Requires updating a broad set of tests across `smoke_e2e.rs`, `integration.rs`, `init_cli_w76.rs`, `cli_comprehensive.rs`, `cli_errors_w66.rs`, and `cli_e2e_w65.rs`.

### Option B
Update the documentation to indicate that `check-ignore` gracefully handles missing files with exit code `0`.
- when to choose it instead: If the priority is avoiding breaking backwards compatibility for scripts that expect `check-ignore` to never fail on missing files.
- trade-offs: Degrades the CI safety guarantees explicitly laid out by the documentation.

## ✅ Decision
Option A was chosen. The documentation correctly identified a UX safety gap (silent CI failures), but the implementation hadn't caught up to the contract. Fixing the codebase aligns the tool with its intended, stricter policy.

## 🧱 Changes made (SRP)
- `crates/tokmd/src/commands/check_ignore.rs`
- `crates/tokmd/tests/cli_comprehensive.rs`
- `crates/tokmd/tests/cli_e2e_w65.rs`
- `crates/tokmd/tests/cli_errors_w66.rs`
- `crates/tokmd/tests/init_cli_w76.rs`
- `crates/tokmd/tests/integration.rs`
- `crates/tokmd/tests/smoke_e2e.rs`

## 🧪 Verification receipts
```text
$ cargo run -- check-ignore does_not_exist.rs
Error: Path 'does_not_exist.rs' does not exist
$ echo $?
1

$ cargo test -p tokmd
test result: ok. 159 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.14s
```

## 🧭 Telemetry
- Change shape: Implementation/Tests
- Blast radius: API (`check-ignore` CLI command). Missing paths will now fail commands rather than silently skipping.
- Risk class: Low
- Rollback: Revert the commit.
- Gates run: `cargo test -p tokmd`, `cargo xtask docs --check`, `cargo xtask version-consistency`

## 🗂️ .jules artifacts
- `.jules/runs/librarian_docs_examples/envelope.json`
- `.jules/runs/librarian_docs_examples/decision.md`
- `.jules/runs/librarian_docs_examples/receipts.jsonl`
- `.jules/runs/librarian_docs_examples/result.json`
- `.jules/runs/librarian_docs_examples/pr_body.md`
- Friction item: `.jules/friction/open/risk-preset-panic.md`

## 🔜 Follow-ups
Logged a friction item for `cargo run -- analyze --preset risk --format md` which panics with "end byte index 4 is not a char boundary; it is inside '日'".
