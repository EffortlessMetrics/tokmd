## 💡 Summary
Added missing property-based invariant tests around `source_complexity.rs`. These invariants explicitly verify that the heuristic rust code parsing and metric aggregations are independent of structural shifts like function re-ordering.

## 🎯 Why
The lightweight heuristic parser in `source_complexity.rs` drives cockpit review gates without pulling in a full AST. It relies on a custom token mask and a state machine tracking bracket depth and function spans. To ensure stable tracking of `total_complexity` and `max_complexity`, we needed property-based verification confirming the aggregation math works commutatively and monotonically regardless of input structure.

## 🔎 Evidence
Minimal proof:
- `crates/tokmd-analysis/src/source_complexity/properties.rs`
- Observed behavior: `source_complexity.rs` was not previously covered by `proptest`. Adding invariants discovered a minor issue with bad generated regexes that we fixed. Both invariants now pass deterministically over randomized input subsets.
- Tests demonstrate: Function order is commutative, and `total >= max` always holds.

## 🧭 Options considered
### Option A (recommended)
- Add property-based invariants directly testing `source_complexity.rs` heuristic aggregations.
- Fits the `analysis-stack` shard, improves property coverage.
- Trade-offs: Increases CI property-testing time slightly, but locks in the core logic.

### Option B
- Try to property-test internal configuration boundaries within `tokmd-gate` (e.g. ratchet definitions).
- Fits the shard, but doesn't find as many structural bugs as generating programmatic inputs against custom parsers.
- Trade-offs: Focuses on JSON serialization edges instead of programmatic metrics logic.

## ✅ Decision
Option A. Adding property-based invariants directly against the `analyze_rust_function_complexity` parser is an honest and high-value proof improvement within the `analysis-stack` that locks in true mathematical properties of the implementation.

## 🧱 Changes made (SRP)
- Added `crates/tokmd-analysis/src/source_complexity/properties.rs`.
- Registered `properties.rs` within `crates/tokmd-analysis/src/source_complexity.rs`.

## 🧪 Verification receipts
```text
> cargo test -p tokmd-analysis properties
...
test source_complexity::properties::tests::property_function_order_independence ... ok
test source_complexity::properties::tests::property_total_gte_max ... ok
...
test result: ok. 103 passed; 0 failed; 0 ignored; 0 measured; 1470 filtered out; finished in 6.47s
```

## 🧭 Telemetry
- Change shape: Add proptest module
- Blast radius: Only test code and metrics behavior tracking
- Risk class: Low
- Rollback: Revert `source_complexity.rs` and delete `properties.rs`
- Gates run: `cargo test`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`, `cargo build --verbose`

## 🗂️ .jules artifacts
- `.jules/runs/invariant_model_analysis/envelope.json`
- `.jules/runs/invariant_model_analysis/decision.md`
- `.jules/runs/invariant_model_analysis/receipts.jsonl`
- `.jules/runs/invariant_model_analysis/result.json`
- `.jules/runs/invariant_model_analysis/pr_body.md`

## 🔜 Follow-ups
None
