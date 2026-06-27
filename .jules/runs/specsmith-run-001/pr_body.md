## 💡 Summary
Added missing BDD scenarios for the `export` command to verify the behavior of the `--children parents-only` and `--children separate` flags.

## 🎯 Why
The `module` and `lang` commands had BDD test coverage for the `--children` flag, but the `export` command lacked these scenarios. This PR fills that gap, locking in the expected behavior and preventing regressions.

## 🔎 Evidence
- `crates/tokmd/tests/bdd_export_scenarios_w50.rs`
- The `export` command parses `--children separate` and `--children parents-only` modes correctly.
- Ran `cargo test -p tokmd --test bdd_export_scenarios_w50` which successfully passed all 10 tests including the two new ones.

## 🧭 Options considered
### Option A (recommended)
- Add BDD tests for `--children parents-only` and `--children separate` to `crates/tokmd/tests/bdd_export_scenarios_w50.rs`.
- Fits well as it explicitly locks in existing CLI parameter behavior in the matching test suite.
- Trade-offs: Minor increase in test suite size, high regression safety.

### Option B
- Add edge case testing for invalid combinations of `--format` and `--meta` in `export`.
- Choose this when edge cases around metadata format handling need coverage.
- Trade-offs: The `--children` flag is more critical since it directly alters dataset row counts and semantics.

## ✅ Decision
Option A. I added BDD scenarios for `--children parents-only` and `--children separate` in `bdd_export_scenarios_w50.rs` to lock in the behavior of the `export` command.

## 🧱 Changes made (SRP)
- `crates/tokmd/tests/bdd_export_scenarios_w50.rs`

## 🧪 Verification receipts
```text
cargo build --verbose
CI=true cargo test -p tokmd --verbose
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test -p tokmd --test bdd_export_scenarios_w50
cargo test -p tokmd --test cli_e2e_w58
cargo test -p tokmd --test cli_pipeline_e2e_w54
```

## 🧭 Telemetry
- Change shape: Test additions
- Blast radius: testing (No production API/IO changes)
- Risk class: Low (Test-only change)
- Rollback: Revert the test additions in `bdd_export_scenarios_w50.rs`
- Gates run: `cargo build`, `cargo test`, `cargo fmt`, `cargo clippy`

## 🗂️ .jules artifacts
- `.jules/runs/specsmith-run-001/envelope.json`
- `.jules/runs/specsmith-run-001/decision.md`
- `.jules/runs/specsmith-run-001/receipts.jsonl`
- `.jules/runs/specsmith-run-001/result.json`
- `.jules/runs/specsmith-run-001/pr_body.md`

## 🔜 Follow-ups
None at this time.
