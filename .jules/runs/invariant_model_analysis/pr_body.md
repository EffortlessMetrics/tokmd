## 💡 Summary
Tightened property-based invariants across ratio calculation surfaces in `tokmd-analysis-api-surface` and `tokmd-analysis-derived`. This explicitly validates that strict rounding matching the underlying mathematical function bounds is consistently applied to both API visibility limits and derived code density scores.

## 🎯 Why
Existing property tests merely verified that boundary ratios were within `[0.0, 1.0]`. They missed verifying that the exact calculation `round_f64(numerator / denominator, 4)` is correctly deterministically preserved across varying generated module tree topologies and mixed-language sets.

## 🔎 Evidence
- `crates/tokmd-analysis-api-surface/tests/properties.rs`
- `crates/tokmd-analysis-derived/tests/properties.rs`

## 🧭 Options considered
### Option A (recommended)
- what it is: Add explicit invariant bound checks verifying the formula `ratio == round(num/denom, 4)` in `properties.rs`.
- why it fits this repo and shard: It locks in the mathematically sound output across proptest generation slices without needing a full mock runner.
- trade-offs: Structure / Velocity / Governance: Low velocity impact, high structural safety.

### Option B
- what it is: Rewrite ratio calculations to use strong typed limits over native floats.
- when to choose it instead: If floats prove to introduce non-deterministic cross-platform behavior.
- trade-offs: Extremely invasive to codebase, unnecessary given `round_f64` mitigations.

## ✅ Decision
Option A was chosen to explicitly encode the mathematical derivation property directly into the target property suites for immediate safety without API churn.

## 🧱 Changes made (SRP)
- Added `exact_mathematical_ratio_invariants` to `crates/tokmd-analysis-api-surface/tests/properties.rs` bounding overall, module, and language ratios mathematically.
- Added `exact_mathematical_ratio_invariants` to `crates/tokmd-analysis-derived/tests/properties.rs` bounding document density and test density ratios dynamically.

## 🧪 Verification receipts
```text
$ cargo test -p tokmd-analysis-api-surface --test properties
test exact_mathematical_ratio_invariants ... ok
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.28s

$ cargo test -p tokmd-analysis-derived --test properties
test exact_mathematical_ratio_invariants ... ok
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.75s
```

## 🧭 Telemetry
- Change shape: New property tests.
- Blast radius: Isolated to property test fixtures.
- Risk class: Low risk. Proof improvement.
- Rollback: Revert the test additions.
- Gates run: `cargo test`

## 🗂️ .jules artifacts
- `.jules/runs/invariant_model_analysis/envelope.json`
- `.jules/runs/invariant_model_analysis/decision.md`
- `.jules/runs/invariant_model_analysis/receipts.jsonl`
- `.jules/runs/invariant_model_analysis/result.json`
- `.jules/runs/invariant_model_analysis/pr_body.md`

## 🔜 Follow-ups
None
