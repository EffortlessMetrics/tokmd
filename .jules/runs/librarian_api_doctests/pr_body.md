## 💡 Summary
Converted `no_run` doctests in core workflows to executable tests and added missing CLI parsing examples. This improves executable test coverage across public APIs to prevent silent drift.

## 🎯 Why
The `interfaces` shard has a `docs-executable` gate profile requiring that examples actually compile and run where possible. The `cockpit_workflow` had a `no_run` example that wasn't verifying the API contract, and the main `Cli` parser lacked an example demonstrating safe parsing (via `try_parse_from`).

## 🔎 Evidence
- `crates/tokmd-core/src/workflows/cockpit.rs` contained `/// ```rust,no_run`.
- `crates/tokmd/src/cli/parser.rs` lacked a struct-level doctest.
- `cargo test -p tokmd --doc` and `cargo test -p tokmd-core --doc` successfully executed the new doctests.

## 🧭 Options considered
### Option A (recommended)
- what it is: Update the existing `no_run` doctest in `cockpit_workflow` to use a temporary git repository via `tempfile` and `std::process::Command`, and add a `try_parse_from` test to `Cli`.
- why it fits this repo and shard: Directly satisfies the `docs-executable` gate profile.
- trade-offs: Structure is preserved. Velocity is slightly impacted by `git init` during doctest execution. Governance is improved through deterministic proof.

### Option B
- what it is: Mock `tokmd-git` behavior for testing.
- when to choose it instead: If git isn't available or takes too long.
- trade-offs: Increases mocking surface area and reduces realism.

## ✅ Decision
Option A. Leveraging a real temporary git repository provides concrete proof of the workflow's correctness without mocking.

## 🧱 Changes made (SRP)
- `crates/tokmd-core/src/workflows/cockpit.rs`: Replaced `no_run` doctest with an executable git-based setup.
- `crates/tokmd/src/cli/parser.rs`: Added an executable doctest to `Cli` demonstrating `try_parse_from`.

## 🧪 Verification receipts
```text
cargo test -p tokmd --doc
test result: ok. 12 passed; 0 failed

cargo test -p tokmd-core --doc --all-features
test result: ok. 14 passed; 0 failed

cargo fmt -- --check && cargo clippy -- -D warnings
Finished `dev` profile
```

## 🧭 Telemetry
- Change shape: Test/docs improvement.
- Blast radius: Docs / tests only.
- Risk class: Low.
- Rollback: Revert PR.
- Gates run: `cargo test --doc`, `cargo fmt`, `cargo clippy`.

## 🗂️ .jules artifacts
- `.jules/runs/librarian_api_doctests/envelope.json`
- `.jules/runs/librarian_api_doctests/decision.md`
- `.jules/runs/librarian_api_doctests/receipts.jsonl`
- `.jules/runs/librarian_api_doctests/result.json`
- `.jules/runs/librarian_api_doctests/pr_body.md`

## 🔜 Follow-ups
None.
