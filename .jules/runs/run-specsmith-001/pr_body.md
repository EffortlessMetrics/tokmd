## 💡 Summary
Added missing BDD scenario tests in `tokmd/tests/bdd_analyze_scenarios_w50.rs` to cover analysis presets: `risk`, `supply`, and `health`.

## 🎯 Why
While the `receipt`, `estimate`, and `fun` presets were covered in integration scenarios, the other analysis-focused presets (`risk`, `supply`, `health`) were missing behavior-level edge-case coverage to ensure their derived metrics (e.g. `complexity`, `assets`, `deps`) correctly appear in the output artifacts when requested.

## 🔎 Evidence
- `crates/tokmd/tests/bdd_analyze_scenarios_w50.rs` lacked tests for `risk`, `supply`, and `health` presets.
- By running `cargo test -p tokmd --test bdd_analyze_scenarios_w50`, it was verified that the new scenarios run successfully and accurately capture expected output behavior.

## 🧭 Options considered
### Option A (recommended)
- what it is: Add BDD scenario tests for the missing analysis presets (`supply`, `risk`, `health`) in `crates/tokmd/tests/bdd_analyze_scenarios_w50.rs`.
- why it fits this repo and shard: Directly fulfills the Specsmith target for "missing BDD/integration coverage for an important path" without altering product-level analysis contracts.
- trade-offs: Structure / Velocity / Governance: Safest, focused change.

### Option B
- what it is: Add deep tests in `tokmd-analysis` checking specific git and complexity edge cases.
- when to choose it instead: If the `preset` parsing itself was flaky or core computation was failing under unique workloads.
- trade-offs: More invasive, changes analysis internal tests rather than pure user-facing behavior tests.

## ✅ Decision
Chose Option A to add scenario coverage. It's clean, improves our regression suite, and directly matches the Specsmith mission constraint.

## 🧱 Changes made (SRP)
- `crates/tokmd/tests/bdd_analyze_scenarios_w50.rs`
  - Added `given_project_when_analyze_risk_then_risk_data_present`
  - Added `given_project_when_analyze_supply_then_supply_data_present`
  - Added `given_project_when_analyze_health_then_health_data_present`

## 🧪 Verification receipts
```text
$ cargo test -p tokmd --test bdd_analyze_scenarios_w50
running 11 tests
test given_project_when_analyze_fun_then_eco_label_present ... ok
test given_project_when_analyze_health_then_health_data_present ... ok
test given_project_when_analyze_json_then_has_args_metadata ... ok
test given_project_when_analyze_estimate_then_effort_model_present ... ok
test given_project_when_analyze_json_then_has_schema_version ... ok
test given_project_when_analyze_md_then_markdown_table ... ok
test given_project_when_analyze_receipt_then_derived_metrics_present ... ok
test given_project_when_analyze_supply_then_supply_data_present ... ok
test given_project_when_analyze_risk_then_risk_data_present ... ok
test given_project_when_analyze_with_output_dir_then_file_created ... ok
test given_project_when_analyze_xml_then_valid_xml_structure ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
```

## 🧭 Telemetry
- Change shape: Tests only.
- Blast radius: Testing layer only.
- Risk class: Low risk. Pure addition of tests.
- Rollback: Revert the test additions.
- Gates run: `cargo clippy`, `cargo fmt`, `cargo test -p tokmd --test bdd_analyze_scenarios_w50`

## 🗂️ .jules artifacts
- `.jules/runs/run-specsmith-001/envelope.json`
- `.jules/runs/run-specsmith-001/decision.md`
- `.jules/runs/run-specsmith-001/receipts.jsonl`
- `.jules/runs/run-specsmith-001/result.json`
- `.jules/runs/run-specsmith-001/pr_body.md`

## 🔜 Follow-ups
None.
