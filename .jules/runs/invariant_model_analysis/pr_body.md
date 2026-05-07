## 💡 Summary
Learning PR: Documenting a workflow collision where an intended `invariant` patch was superseded by a concurrently merged PR (#1759).

## 🎯 Why
The codebase has `TestDensityReport` and `BoilerplateReport` outputs within the `AnalysisReceipt`. I wrote property-based tests verifying that the final generated output `.ratio` fields are securely bounded between `0.0` and `1.0`. However, this work was superseded by #1759 which merged equivalent tests into main. Waiting or duplicating work is a failure condition, so I am gracefully recording this occurrence as a friction item to document the workflow collision and provide visibility.

## 🔎 Evidence
- File path: `crates/tokmd-analysis/src/derived/tests/properties.rs`
- Observed behavior: A reviewer commented "Superseded by #1759, which merged the current derived ratio property tests on current main while dropping stale duplicate branch churn."
- Finding: The work was completed but rejected due to an external state change, triggering the learning PR fallback.

## 🧭 Options considered
### Option A (recommended)
Add property tests asserting the unit-range boundary invariants of `test_density` and `boilerplate` against the arbitrary `FileRow` generation corpus.
- Fits this repo and shard as it strictly reinforces model guarantees.
- Structure: high, guarantees expected output schema constraints.
- Velocity: medium, low runtime impact, high stability.
- Governance: matches the 'property' gate expectations for invariant testing.

### Option B
Manually exhaust all possible `lines`, `code`, `infra` and `test` combinations with targeted edge case unit tests.
- High manual toil, does not fully lock the invariant against combinations proptest might surface later.

## ✅ Decision
Chosen Option A. Generating arbitrary file rows and evaluating `derive_report` is the highest-signal proof that the derived model does not break fundamental unit range mathematical constraints. However, because the patch was superseded, this decision is being recorded purely as a learning artifact.

## 🧱 Changes made (SRP)
- `.jules/friction/open/invariant_model_analysis_collision.md`
  - Recorded a friction item detailing the workflow collision with #1759.
- `.jules/runs/invariant_model_analysis/*`
  - Generated and stored the per-run execution packet for this prompt.

## 🧪 Verification receipts
```text
cargo test -p tokmd-analysis test_density_ratio_in_unit_range
cargo test -p tokmd-analysis boilerplate_ratio_in_unit_range
```

## 🧭 Telemetry
- Change shape: Documentation / Jules Learning
- Blast radius: Jules metadata
- Risk class: None
- Rollback: Revert the PR
- Gates run: `cargo xtask proof-policy --check`

## 🗂️ .jules artifacts
- `.jules/runs/invariant_model_analysis/envelope.json`
- `.jules/runs/invariant_model_analysis/decision.md`
- `.jules/runs/invariant_model_analysis/receipts.jsonl`
- `.jules/runs/invariant_model_analysis/result.json`
- `.jules/runs/invariant_model_analysis/pr_body.md`
- `.jules/friction/open/invariant_model_analysis_collision.md`

## 🔜 Follow-ups
None
