## 💡 Summary
Adds an end-to-end BDD scenario to verify the Health preset correctly triggers and reports TODO metrics when scanning real files. This closes an integration coverage gap.

## 🎯 Why
The `Health` preset explicitly requests the `todo` and `complexity` features, but the test suite lacked a BDD-style end-to-end integration test verifying that the pipeline correctly populates `todo` metrics when scanning real files under this preset. Adding this locks in the expected behavior and prevents regressions where the preset is requested but the underlying scanner fails silently.

## 🔎 Evidence
- `crates/tokmd-analysis/tests/analysis_deep_w64.rs`
- Missing BDD coverage for the `Health` preset TODO metrics.
- Verified by running `cargo test -p tokmd-analysis bdd_health`.

## 🧭 Options considered
### Option A (recommended)
- Add BDD coverage for the `Health` preset focusing on its derivation of TODO metrics, validating that its end-to-end integration with the content module correctly populates metrics when files are scanned.
- Matches Specsmith's mission to improve scenario coverage.
- Trade-offs: Minor increase in test suite duration due to temporary file I/O for `todo` scanning.

### Option B
- Do not add the test, leaving the end-to-end `Health` preset TODO scanning untested at the integration level.
- Saves minor I/O cost in tests.
- Trade-offs: Leaves a gap in regression coverage around feature-gated behavior inside an important preset.

## ✅ Decision
Selected Option A. It explicitly fills an integration gap and locks in the expected behavior of the Health preset against real file scanning.

## 🧱 Changes made (SRP)
- Added `bdd_health_preset_includes_todo_metrics` in `crates/tokmd-analysis/tests/analysis_deep_w64.rs`.

## 🧪 Verification receipts
```text
running 1 test
test bdd_health_preset_includes_todo_metrics ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 66 filtered out; finished in 0.00s
```

## 🧭 Telemetry
- Change shape: New BDD test
- Blast radius: Tests only
- Risk class: Low
- Rollback: `git checkout -- crates/tokmd-analysis/tests/analysis_deep_w64.rs`
- Gates run: `cargo test -p tokmd-analysis`

## 🗂️ .jules artifacts
- `.jules/runs/specsmith_analysis_stack/envelope.json`
- `.jules/runs/specsmith_analysis_stack/decision.md`
- `.jules/runs/specsmith_analysis_stack/receipts.jsonl`
- `.jules/runs/specsmith_analysis_stack/result.json`
- `.jules/runs/specsmith_analysis_stack/pr_body.md`

## 🔜 Follow-ups
None.
