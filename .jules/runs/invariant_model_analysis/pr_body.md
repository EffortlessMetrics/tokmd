## 💡 Summary
Added two new property-based tests in `tokmd-analysis` to explicitly lock in the unit range invariant `[0.0, 1.0]` for `test_density.ratio` and `boilerplate.ratio`.

## 🎯 Why
The codebase has `TestDensityReport` and `BoilerplateReport` outputs within the `AnalysisReceipt`. These reports calculate ratios based on code logic and infrastructure lines. By mathematically defining boundaries around these metric values, we strengthen the reliability of our reports across any potential combination of inputs. Missing property tests for these left edge behavior unverified against extreme values surfaced by proptest.

## 🔎 Evidence
- File path: `crates/tokmd-analysis/src/derived/tests/properties.rs`
- Finding: `test_density` and `boilerplate` metrics lacked property-based test invariants verifying that the final generated output `.ratio` fields are securely bounded between `0.0` and `1.0`.

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
Chosen Option A. Generating arbitrary file rows and evaluating `derive_report` is the highest-signal proof that the derived model does not break fundamental unit range mathematical constraints.

## 🧱 Changes made (SRP)
- `crates/tokmd-analysis/src/derived/tests/properties.rs`
  - Added `test_density_ratio_in_unit_range`
  - Added `boilerplate_ratio_in_unit_range`

## 🧪 Verification receipts
```text
cargo test -p tokmd-analysis test_density_ratio_in_unit_range
cargo test -p tokmd-analysis boilerplate_ratio_in_unit_range
```

## 🧭 Telemetry
- Change shape: Internal tests only
- Blast radius: `tokmd-analysis` tests
- Risk class: Low
- Rollback: Revert the PR
- Gates run: `cargo test -p tokmd-analysis`

## 🗂️ .jules artifacts
- `.jules/runs/invariant_model_analysis/envelope.json`
- `.jules/runs/invariant_model_analysis/decision.md`
- `.jules/runs/invariant_model_analysis/receipts.jsonl`
- `.jules/runs/invariant_model_analysis/result.json`
- `.jules/runs/invariant_model_analysis/pr_body.md`

## 🔜 Follow-ups
None
