## 💡 Summary
Added BDD-style regression tests to verify CLI error paths correctly distinguish between an unrecognized subcommand (like `anolyze`) and a missing file path (like `missing/dir`).

## 🎯 Why
CLI edge-case behaviors (such as identifying missing paths vs. identifying typos of known subcommands) are managed by error hinting (`error_hints.rs`), but the BDD integration test suite (`bdd_scenarios_w71.rs` and its siblings) lacked clear scenario coverage. This proof-improvement patch explicitly locks in the exact behavior around ambiguous path inputs, improving regression coverage for the `interfaces` shard.

## 🔎 Evidence
- `crates/tokmd/tests/cli_e2e_w65.rs` and `cli_error_paths_w51.rs` verified fragments of this logic with unit-like assertions, but high-level BDD scenario coverage was missing.
- Observed that running `tokmd anolyze` falls back to `lang` subcommand, returning a missing path error which then gets correctly transformed to a 'Did you mean' message via `error_hints::format`.
- Ran `cargo test err_typo_subcommand_fails` to trace existing test cases.

## 🧭 Options considered
### Option A (recommended)
- Add a dedicated BDD test file `crates/tokmd/tests/bdd_error_paths_w71.rs` covering CLI error reporting for unrecognized tokens and paths.
- Fits this repo and shard as it improves BDD/scenario coverage around `tokmd`'s CLI error formatting.
- Trade-offs: Minor addition to test structure; provides excellent regression proofing with good velocity without governance friction.

### Option B
- Embed the new BDD scenarios into an existing functional test file like `bdd_scenarios_w71.rs`.
- When to choose: if reducing the file count is preferred.
- Trade-offs: Clutters the "happy path" scenarios with explicit error-condition scenarios.

## ✅ Decision
Proceed with Option A. A dedicated test file keeps error-condition behaviors cleanly isolated from standard workflows, fulfilling the Specsmith mission of locking in edge cases.

## 🧱 Changes made (SRP)
- Added `crates/tokmd/tests/bdd_error_paths_w71.rs`

## 🧪 Verification receipts
```text
$ cargo test --test bdd_error_paths_w71
running 2 tests
test given_path_looking_string_when_run_then_reports_as_path_not_found ... ok
test given_unrecognized_bare_token_when_run_then_reports_as_unrecognized_subcommand ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

## 🧭 Telemetry
- Change shape: New test file
- Blast radius: Testing-only (Integration tests)
- Risk class: Safe (No production code changes)
- Rollback: Trivial (revert PR)
- Gates run: `cargo build --verbose`, `CI=true cargo test --verbose`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`

## 🗂️ .jules artifacts
- `.jules/runs/specsmith_interfaces_01/envelope.json`
- `.jules/runs/specsmith_interfaces_01/decision.md`
- `.jules/runs/specsmith_interfaces_01/receipts.jsonl`
- `.jules/runs/specsmith_interfaces_01/result.json`
- `.jules/runs/specsmith_interfaces_01/pr_body.md`

## 🔜 Follow-ups
None.
