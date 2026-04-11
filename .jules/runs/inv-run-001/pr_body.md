## 💡 Summary
Added a new `properties_w60.rs` file for `tokmd-analysis-api-surface` with property-based tests verifying the extraction logic deterministically tracks total and public Rust symbols under randomly generated file conditions, and handles duplication and missing files gracefully.

## 🎯 Why
The `build_api_surface_report` and `extract_rust_symbols` functions are central model code with edge cases around missing files, duplicate inputs, and symbol combinations. Testing them deeply with proptest generated inputs improves our confidence in the invariant that token counts perfectly mirror valid structural language tokens.

## 🔎 Evidence
Minimal proof:
- Added `crates/tokmd-analysis-api-surface/tests/properties_w60.rs`
- Proptests `exact_item_count_matches`, `prop_missing_files_graceful`, and `prop_duplicate_files_are_processed` successfully assert extraction properties against `build_api_surface_report`.

## 🧭 Options considered
### Option A
- what it is: Add a proptest for invalid language
- why it fits this repo and shard: Validates unsupported languages.
- trade-offs: Minimal gain compared to structural extraction invariant validation.

### Option B (recommended)
- what it is: Add a new proptest module asserting structural extraction correctness in Rust files
- when to choose it instead: Tightens property-based tests around real invariants in the `tokmd-analysis-api-surface` crate.
- trade-offs: Requires generating realistic code combinations.

## ✅ Decision
Decided to implement Option B. Added property tests asserting the total count exactly matches permutations from generated rust symbols, and added tests for duplicate and missing files.

## 🧱 Changes made (SRP)
- Created `crates/tokmd-analysis-api-surface/tests/properties_w60.rs`

## 🧪 Verification receipts
```text
cargo test -p tokmd-analysis-api-surface properties_w60
# Success, 3 tests passed.
```

## 🧭 Telemetry
- Change shape: New test file
- Blast radius: None (Test-only change)
- Risk class: Low
- Rollback: Revert commit
- Gates run: `cargo test -p tokmd-analysis-api-surface`

## 🗂️ .jules artifacts
- `.jules/runs/inv-run-001/envelope.json`
- `.jules/runs/inv-run-001/decision.md`
- `.jules/runs/inv-run-001/receipts.jsonl`
- `.jules/runs/inv-run-001/result.json`
- `.jules/runs/inv-run-001/pr_body.md`

## 🔜 Follow-ups
None.
