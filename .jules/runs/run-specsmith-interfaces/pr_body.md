## 💡 Summary
Added a BDD scenario integration test for the `risk` preset to ensure it maintains the same level of coverage as other primary presets (`health`, `receipt`).

## 🎯 Why
The BDD scenarios in `crates/tokmd/tests/bdd_analyze_scenarios_w50.rs` lock in the CLI contracts for the `analyze` command. While `health`, `receipt`, `estimate`, and `fun` presets were covered, `risk` (which importantly triggers git history analysis alongside health) was missing. Ensuring a failing `git` traversal or misconfigured risk preset breaks a core BDD test prevents silent regressions.

## 🔎 Evidence
- `crates/tokmd/tests/bdd_analyze_scenarios_w50.rs`
- Missing coverage for `--preset risk` despite it being one of the core presets defined in `crates/tokmd/src/cli/parser/analysis.rs`.

## 🧭 Options considered
### Option A (recommended)
- Add a targeted BDD scenario test for `tokmd analyze --preset risk` to the existing suite.
- Fits the `Specsmith` mission perfectly by improving scenario coverage and locking in the edge-case behavior of the git traversal in integration tests.
- High velocity, safe structure, low governance overhead.

### Option B
- Clean up the duplicated error path tests (`analyze_invalid_preset_fails`) across `cli_error_paths_w51.rs`, `cli_errors_w66.rs`, and `error_handling.rs`.
- Do not choose this because doing so violates the anti-drift rule: "Do not become a generic test cleanup lane."

## ✅ Decision
Option A. It adds concrete proof coverage for an important but uncovered CLI path, directly aligning with Specsmith's target ranking.

## 🧱 Changes made (SRP)
- `crates/tokmd/tests/bdd_analyze_scenarios_w50.rs`: Added `given_project_when_analyze_risk_then_risk_metrics_present` test.

## 🧪 Verification receipts
```text
test given_project_when_analyze_risk_then_risk_metrics_present ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
```

## 🧭 Telemetry
- Change shape: Test addition.
- Blast radius: `crates/tokmd/tests/` (Test suite only).
- Risk class: Low (Additive test).
- Rollback: Revert the added test in `crates/tokmd/tests/bdd_analyze_scenarios_w50.rs`.
- Gates run: `core-rust` fallback gate (cargo fmt, clippy, test).

## 🗂️ .jules artifacts
- `.jules/runs/run-specsmith-interfaces/envelope.json`
- `.jules/runs/run-specsmith-interfaces/decision.md`
- `.jules/runs/run-specsmith-interfaces/receipts.jsonl`
- `.jules/runs/run-specsmith-interfaces/result.json`
- `.jules/runs/run-specsmith-interfaces/pr_body.md`

## 🔜 Follow-ups
None.
