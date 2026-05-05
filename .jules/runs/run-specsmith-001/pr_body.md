## 💡 Summary
This is a learning PR. The previously attempted patch adding BDD test scenarios for analysis presets (`risk`, `supply`, `health`) was closed as redundant, as it duplicated existing coverage in the `health` pipeline proof merged in #1578.

## 🎯 Why
During the run, broad presence-check BDD scenarios were added to `tokmd/tests/bdd_analyze_scenarios_w50.rs`. Upon review, it was discovered that this duplicated coverage already merged into the codebase. To adhere to prompt-to-PR pipeline constraints and prevent abandoned tasks, this run is finalized as a Learning PR capturing the friction edge case.

## 🔎 Evidence
- `tokmd/tests/bdd_analyze_scenarios_w50.rs` modifications were attempted.
- Reviewer feedback: "Reviewed in the analysis invariant cluster. Closing rather than merging: these broad analyze-preset key-presence checks duplicate current CLI/preset coverage and are weaker than the focused fixture-backed Health pipeline proof merged in #1578."

## 🧭 Options considered
### Option A
- what it is: Add BDD scenario tests for the missing analysis presets (`supply`, `risk`, `health`) in `crates/tokmd/tests/bdd_analyze_scenarios_w50.rs`.
- when to choose it instead: If the `preset` parsing itself was flaky or core computation was failing under unique workloads without existing coverage.
- trade-offs: Redundant in this context.

### Option B (recommended)
- what it is: Abandon the redundant BDD test patch since it was superseded by #1578, and record a Learning PR covering this workflow friction edge case.
- why it fits this repo and shard: Directly fulfills the Specsmith constraint to produce a learning PR containing a friction item when a valid code patch is not justified.
- trade-offs: Structure / Velocity / Governance: Safest, focused change that accurately reflects the outcome.

## ✅ Decision
Chose Option B to create a Learning PR and record the friction item.

## 🧱 Changes made (SRP)
- `.jules/friction/open/specsmith_duplicate_analysis_tests.md`
- `.jules/runs/run-specsmith-001/envelope.json`
- `.jules/runs/run-specsmith-001/decision.md`
- `.jules/runs/run-specsmith-001/receipts.jsonl`
- `.jules/runs/run-specsmith-001/result.json`
- `.jules/runs/run-specsmith-001/pr_body.md`

## 🧪 Verification receipts
```text
(Run gracefully aborted post-review due to redundant scope.)
```

## 🧭 Telemetry
- Change shape: Learning PR only.
- Blast radius: None.
- Risk class: Low risk.
- Rollback: Revert the artifacts.
- Gates run: None.

## 🗂️ .jules artifacts
- `.jules/runs/run-specsmith-001/envelope.json`
- `.jules/runs/run-specsmith-001/decision.md`
- `.jules/runs/run-specsmith-001/receipts.jsonl`
- `.jules/runs/run-specsmith-001/result.json`
- `.jules/runs/run-specsmith-001/pr_body.md`
- Added friction item: `.jules/friction/open/specsmith_duplicate_analysis_tests.md`

## 🔜 Follow-ups
None.
