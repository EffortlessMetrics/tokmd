## 💡 Summary
Tightened property invariants around derived analysis distributions and histograms. These new proptests verify the correctness of the median, percentiles (p90, p99), and ensure that histogram buckets remain contiguous without overlaps or gaps.

## 🎯 Why
Previously, the derived properties only verified the extremes (min, max, mean, gini) of the distribution, ignoring the core statistical quantiles and assuming bucket boundaries without checking their coherence across sizes. Locking these down prevents regressions in report derivations.

## 🔎 Evidence
- `crates/tokmd-analysis/src/derived/tests/properties.rs`
- Observed missing tests for `median`, `p90`, `p99`, and bucket boundary contiguity.
- Proptest coverage now properly models these bounds and constraints.

## 🧭 Options considered
### Option A (recommended)
- Add strict checks for median calculations and gapless bounds between `min`/`max` fields on histogram buckets.
- Fits the analysis-stack by increasing deterministic guarantees in data generation.
- **Trade-offs:** Increases test compilation and execution time slightly but greatly enhances proof surface for `distribution_report` structure.

### Option B
- Add deterministic hard-coded unit test values.
- **When to choose:** Only when randomized testing fails to hit specific complex mathematical edge cases or requires deterministic manual coverage.
- **Trade-offs:** Weaker invariants and more maintenance overhead over time.

## ✅ Decision
Chose Option A to enforce invariants across arbitrary file arrays properly.

## 🧱 Changes made (SRP)
- `crates/tokmd-analysis/src/derived/tests/properties.rs` - added `distribution_median_is_correct`, `distribution_percentiles_are_correct`, and `histogram_bucket_bounds_are_ordered_and_non_overlapping`.

## 🧪 Verification receipts
```text
cargo test -p tokmd-analysis --lib derived::tests::properties
...
test derived::tests::properties::distribution_median_is_correct ... ok
test derived::tests::properties::distribution_percentiles_are_correct ... ok
test derived::tests::properties::histogram_bucket_bounds_are_ordered_and_non_overlapping ... ok
...
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 1515 filtered out; finished in 4.40s
```

## 🧭 Telemetry
- Change shape: Test/Proof improvement
- Blast radius: Internal test surface only
- Risk class: Low
- Rollback: Revert the PR
- Gates run: `cargo test -p tokmd-analysis --lib derived::tests::properties`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`

## 🗂️ .jules artifacts
- `.jules/runs/invariant_model_analysis/envelope.json`
- `.jules/runs/invariant_model_analysis/decision.md`
- `.jules/runs/invariant_model_analysis/receipts.jsonl`
- `.jules/runs/invariant_model_analysis/result.json`
- `.jules/runs/invariant_model_analysis/pr_body.md`

## 🔜 Follow-ups
None.
