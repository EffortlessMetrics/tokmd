## 💡 Summary
Extracted `sort_lang_rows`, `sort_module_rows`, and `sort_file_rows` into public functions within `tokmd-model`. This allows the property-based determinism tests to test the exact sorting logic used by the core aggregation functions without needing to duplicate the closures within the tests themselves.

## 🎯 Why
The `tokmd-model` determinism property tests (`determinism_w66.rs`) were previously redefining sorting logic using local closures because the actual `sort_by` sorting loops were inlined within `create_lang_report_from_rows`, `create_module_report_from_rows`, and `create_export_data_from_rows`. Exposing these as standalone public functions directly tightens property-based tests around the real invariants and eliminates redundant test-local closures.

## 🔎 Evidence
- `crates/tokmd-model/src/lib.rs`
- `crates/tokmd-model/tests/determinism_w66.rs`

## 🧭 Options considered
### Option A (recommended)
- Extract `sort_lang_rows`, `sort_module_rows`, and `sort_file_rows` as public functions.
- **Why it fits**: Directly satisfies the Invariant profile memory pointing out the lack of public sorting functions for determinism testing.
- **Trade-offs**: Improves DRY and makes the public API more testable without redundant internal closures in the test file.

### Option B
- Keep the inline closures and test determinism exclusively through the parent report builder functions.
- **When to choose**: If exporting the sorting logic clutters the public API unnecessarily.
- **Trade-offs**: Makes writing and maintaining targeted property testing on sort invariants significantly more cumbersome.

## ✅ Decision
Option A. It explicitly tightens property test invariants as required by the `Invariant` persona instructions for `tokmd-model` and effectively targets the friction mentioned in memory.

## 🧱 Changes made (SRP)
- `crates/tokmd-model/src/lib.rs`
- `crates/tokmd-model/tests/determinism_w66.rs`

## 🧪 Verification receipts
```text
running 20 tests
test avg_function_determinism ... ok
test btreemap_duplicate_entries_aggregate_deterministically ... ok
test btreemap_aggregation_is_deterministic ... ok
test children_mode_collapse_report_is_stable ... ok
test children_mode_separate_report_is_stable ... ok
test export_data_sort_code_desc_then_path_asc ... ok
test file_rows_insertion_order_does_not_affect_sorted_output ... ok
test module_key_is_path_order_independent ... ok
test lang_rows_insertion_order_does_not_affect_sorted_output ... ok
test module_key_normalizes_separators ... ok
test module_rows_insertion_order_does_not_affect_sorted_output ... ok
test normalize_path_determinism ... ok
test normalize_path_forward_slash_consistency ... ok
test module_report_child_include_mode_serialization_stable ... ok
test prop_file_sort_any_permutation ... ok
test prop_module_sort_any_permutation ... ok
test tie_breaking_by_name_is_deterministic ... ok
test top_n_folding_is_deterministic ... ok
test prop_lang_sort_any_permutation ... ok
test prop_btreemap_ordering_is_lexicographic ... ok

test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
```

## 🧭 Telemetry
- Change shape: Extracted logic + test update
- Blast radius: Low risk; internal logic identical, only API visibility increased for tests.
- Risk class + why: Low. The sorting invariants remain identical, strictly improving code quality.
- Rollback: Revert changes in `src/lib.rs` and `determinism_w66.rs`.
- Gates run: `cargo test -p tokmd-model`, `cargo fmt`, `cargo clippy`.

## 🗂️ .jules artifacts
- `.jules/runs/invariant_model_analysis/envelope.json`
- `.jules/runs/invariant_model_analysis/decision.md`
- `.jules/runs/invariant_model_analysis/receipts.jsonl`
- `.jules/runs/invariant_model_analysis/result.json`
- `.jules/runs/invariant_model_analysis/pr_body.md`

## 🔜 Follow-ups
None.
