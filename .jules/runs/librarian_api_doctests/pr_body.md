## 💡 Summary
This is a learning PR. Made the `analysis_facade` doctest in `tokmd-core` executable to prevent silent drift, and recorded a friction item detailing why `cockpit_workflow` cannot easily be made executable without complex Git mocking.

## 🎯 Why
Several doctests in `tokmd-core` (like `cockpit_workflow` and `analysis_facade`) were marked with `no_run` which skips validation, potentially hiding compilation and runtime drift. This violates the `docs-executable` gate profile expectation. While `analysis_facade` was straightforward to fix, `cockpit_workflow` fails during `cargo test` because it implicitly relies on being executed inside an active Git repository.

## 🔎 Evidence
- `crates/tokmd-core/src/lib.rs` had `rust,no_run` applied to multiple facade and workflow routines.
- Running `cockpit_workflow` in a test environment without a properly initialized Git repository causes it to error with "not inside a git repository", leading to a test failure if `no_run` is removed.

## 🧭 Options considered
### Option A
- Wrap the `cockpit_workflow` assertion in an `if let Ok(receipt) = ...` block to swallow the Git error.
- Trade-offs: This is an anti-pattern. If the workflow fails due to missing Git context in CI, the test silently passes without executing the assertions, which is functionally no better than `no_run`.

### Option B (recommended)
- Fix what can be cleanly executed (`analysis_facade`) and document the `cockpit_workflow` Git dependency issue as a friction item so that a proper mocking or temporary directory fixture solution can be implemented later.
- Fits the `interfaces` shard properly without compromising test integrity.

## ✅ Decision
Selected Option B. We replaced `no_run` instances in `tokmd-core` for `analysis_facade` which has no external dependencies. We documented the `cockpit_workflow` issue as friction to avoid swallowing errors in tests.

## 🧱 Changes made (SRP)
- `crates/tokmd-core/src/lib.rs`: Removed `no_run` from `analysis_facade`.

## 🧪 Verification receipts
```text
{"command": "python3 fix_core.py", "outcome": "Success - replaced no_run with rust for analysis_facade in tokmd-core/src/lib.rs"}
{"command": "cargo test -p tokmd-core --doc", "outcome": "Success (6 passed)"}
```

## 🧭 Telemetry
- Change shape: Proof improvement and Learning.
- Blast radius: API docs. Zero production logic altered.
- Risk class: Low risk. Proof improvement.
- Rollback: Safe to revert.
- Gates run: `cargo test -p tokmd-core --doc`.

## 🗂️ .jules artifacts
- `.jules/runs/librarian_api_doctests/envelope.json`
- `.jules/runs/librarian_api_doctests/decision.md`
- `.jules/runs/librarian_api_doctests/receipts.jsonl`
- `.jules/runs/librarian_api_doctests/result.json`
- `.jules/runs/librarian_api_doctests/pr_body.md`
- `.jules/friction/open/librarian_doctest_git_dependency.md`

## 🔜 Follow-ups
- See friction item regarding `cockpit_workflow` needing a robust way to mock Git state in doctests.
