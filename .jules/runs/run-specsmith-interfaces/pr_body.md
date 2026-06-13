## 💡 Summary
Added BDD-style documentation tests for `tokmd gate` and `tokmd cockpit` commands. This locks in the Given/When/Then behaviour for Ratchet CI failure boundaries and the Cockpit PR diff analysis output.

## 🎯 Why
While both the `gate` and `cockpit` commands had unit and integration coverage, they were missing the `bdd_*_scenarios_w50.rs` structural coverage that the other commands (`lang`, `export`, `diff`, `analyze`) possessed. This gap made the exact user-facing contract around CI ratchet failures and PR Markdown generation harder to reason about and vulnerable to regression drift.

## 🔎 Evidence
- `crates/tokmd/tests/bdd_gate_scenarios_w50.rs` (created)
- `crates/tokmd/tests/bdd_cockpit_scenarios_w50.rs` (created)
- `cargo test -p tokmd --test bdd_gate_scenarios_w50` passes
- `cargo test -p tokmd --test bdd_cockpit_scenarios_w50` passes

## 🧭 Options considered
### Option A (recommended)
- Add BDD tests for the `gate` and `cockpit` CLI commands.
- Why it fits: The primary shard is `interfaces`, and the persona is Specsmith. This is a targeted improvement to the integration test boundary (which is an allowed path).
- Trade-offs: Increases CI runtime slightly but adds strong structural regression protection.

### Option B
- Refactor the existing CLI integration tests (`gate_integration.rs` and `cockpit_cli_w71.rs`) to be pure BDD files.
- When to choose it instead: If the existing test files were already 90% BDD.
- Trade-offs: Carries a high risk of losing non-scenario mechanical coverage hidden deep within the existing files. Additive BDD testing is safer.

## ✅ Decision
Chose Option A to create additive Given/When/Then proofs for the highest-value edge cases (Ratchet behaviour and PR Diff generation) to satisfy the Specsmith goal.

## 🧱 Changes made (SRP)
- Added `crates/tokmd/tests/bdd_gate_scenarios_w50.rs`
- Added `crates/tokmd/tests/bdd_cockpit_scenarios_w50.rs`

## 🧪 Verification receipts
```text
running 3 tests
test given_missing_baseline_when_gate_evaluated_then_it_errors ... ok
test given_degraded_metrics_when_gate_evaluated_then_gate_fails ... ok
test given_improved_metrics_when_gate_evaluated_then_gate_passes ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

running 2 tests
test given_git_branch_with_changes_when_cockpit_md_then_sections_generated ... ok
test given_git_branch_with_changes_when_cockpit_json_then_valid_schema ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
```

## 🧭 Telemetry
- Change shape: proof-improvement patch
- Blast radius: Testing only. No IO/API changes.
- Risk class: Low - additive integration tests.
- Rollback: Revert the added test files.
- Gates run: `core-rust` (cargo build, test, fmt, clippy).

## 🗂️ .jules artifacts
- `.jules/runs/run-specsmith-interfaces/envelope.json`
- `.jules/runs/run-specsmith-interfaces/decision.md`
- `.jules/runs/run-specsmith-interfaces/receipts.jsonl`
- `.jules/runs/run-specsmith-interfaces/result.json`
- `.jules/runs/run-specsmith-interfaces/pr_body.md`

## 🔜 Follow-ups
None.
