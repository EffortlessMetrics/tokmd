## 💡 Summary
Replaced vague stderr assertions in CLI error tests with explicit text matches. This locks in the actual error strings being emitted rather than just verifying that stderr wasn't empty.

## 🎯 Why
The Specsmith persona focuses on scenario coverage and regression prevention. Using `.stderr(predicate::str::is_empty().not())` is an anti-pattern because it allows error messages to degrade or change entirely without failing the test suite. By matching specific hints (e.g., `"Path not found"`, `"Unrecognized subcommand"`), we lock in deterministic error-handling behavior.

## 🔎 Evidence
- `crates/tokmd/tests/cli_error_paths_w51.rs`
- `crates/tokmd/tests/cli_errors_w66.rs`
- `crates/tokmd/tests/error_handling_w70.rs`
- Observed behavior: Tests were accepting any non-empty stderr string, which hides potential regressions in error hinting.

## 🧭 Options considered
### Option A (recommended)
- Polish and tighten assertion specificity in error/help CLI tests by changing vague `.stderr(predicate::str::is_empty().not())` to explicit checks for known hints like `"invalid value"`, `"Path not found"`, etc.
- Fits the repo and shard because the assignment explicitly calls for "scenario-driven sharp-edge polish" and memory highlights vague stderr predicates as an anti-pattern.
- Trade-offs: Structure is better locked-in; Velocity is high; Governance matches the Specsmith persona perfectly.

### Option B
- Investigate edge cases in `handoff` context strategies, e.g., finding edge-cases that are not covered around budget tokens limit and strategy combinations.
- Choose instead if there is a clear missing edge-case in budget parsing or boundary limits that is not asserted.
- Trade-offs: Requires deep digging to find a legitimate uncovered path, which might not exist or might require cross-shard changes.

## ✅ Decision
I chose **Option A**. The memory specifically notes: `When writing or updating CLI integration tests in the tokmd workspace, avoid using vague assertions like .stderr(predicate::str::is_empty().not()). Instead, use explicit .stderr(predicates::str::contains("...")) assertions to strictly validate specific error hints and subcommand suggestions.` Finding instances of this in W51, W66, and W70 tests and replacing them with specific assertions was an actionable and valuable improvement.

## 🧱 Changes made (SRP)
- `crates/tokmd/tests/cli_error_paths_w51.rs`
- `crates/tokmd/tests/cli_errors_w66.rs`
- `crates/tokmd/tests/error_handling_w70.rs`

## 🧪 Verification receipts
```text
cargo test -p tokmd --test cli_error_paths_w51
cargo test -p tokmd --test cli_errors_w66
cargo test -p tokmd --test error_handling_w70
# All tests passed.
```

## 🧭 Telemetry
- Change shape: Proof-improvement patch
- Blast radius: Tests only.
- Risk class: Low risk; no logic changes, only test assertion tightening.
- Rollback: Revert the PR.
- Gates run: `cargo test` on affected crates.

## 🗂️ .jules artifacts
- `.jules/runs/run-specsmith-1/envelope.json`
- `.jules/runs/run-specsmith-1/decision.md`
- `.jules/runs/run-specsmith-1/receipts.jsonl`
- `.jules/runs/run-specsmith-1/result.json`
- `.jules/runs/run-specsmith-1/pr_body.md`

## 🔜 Follow-ups
None.
