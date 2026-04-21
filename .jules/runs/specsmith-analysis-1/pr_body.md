## 💡 Summary
Updated the `is_test_path` heuristic in `tokmd-analysis-util` to correctly identify standalone test files (e.g., `test.rs`, `spec.js`, `tests.py`).

## 🎯 Why
Previously, the heuristic primarily relied on directory names (`/test/`, `/tests/`, `/spec/`, etc.) or specific prefixes/suffixes (`test_*.rs`, `*_test.rs`, `*.test.js`). It completely missed standalone files at the root or within standard directories that were simply named `test.rs` or `spec.js`. This resulted in inaccurate test density analysis, an important edge-case regression gap.

## 🔎 Evidence
- **File:** `crates/tokmd-analysis-util/src/lib.rs`
- **Observed Behavior:** Calling `is_test_path("test.rs")` returned `false`.
- **Receipt:** Before changes, `is_test_path_edge_cases::given_file_named_test_without_underscore_then_not_detected_as_file_pattern` was passing by asserting `!result`. Now, testing files properly correctly returns `true`.

## 🧭 Options considered
### Option A (recommended)
- **What it is:** Update `is_test_path` logic to accept files starting with `test.`, `tests.`, `spec.`, and `specs.`.
- **Why it fits:** It directly addresses the coverage gap while maintaining deterministic heuristic matching.
- **Trade-offs:** Structure is clean, slightly higher logic but minimal performance impact.

### Option B
- **What it is:** Leave the logic as is and require users to put test files in specific directories or rename them.
- **When to choose it:** If we want strict control over test file naming conventions.
- **Trade-offs:** Causes confusion for users with standard test file names in ecosystems like Python (`tests.py`) and JS (`spec.js`).

## ✅ Decision
Option A. It's an important edge-case regression/gap not locked in by tests, perfectly aligning with Specsmith's mission to improve scenario coverage and regression coverage.

## 🧱 Changes made (SRP)
- `crates/tokmd-analysis-util/src/lib.rs`: Updated `is_test_path` string matching.
- `crates/tokmd-analysis-util/tests/bdd.rs`: Added BDD scenarios for standalone test files.
- `crates/tokmd-analysis-util/tests/bdd_expansion.rs`: Adjusted previous expectation that `test.rs` wouldn't be matched.

## 🧪 Verification receipts
```text
test result: ok. 59 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s (tokmd-analysis-util)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s (tokmd-analysis-derived)
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s (tokmd-analysis)
```

## 🧭 Telemetry
- Change shape: Logic extension
- Blast radius: API schema unchanged. Heuristics for analysis will identify slightly more files as tests.
- Risk class: Low risk. Fixes a clear false negative.
- Rollback: Revert `lib.rs` string checks.
- Gates run: `cargo test`

## 🗂️ .jules artifacts
- `.jules/runs/specsmith-analysis-1/envelope.json`
- `.jules/runs/specsmith-analysis-1/decision.md`
- `.jules/runs/specsmith-analysis-1/receipts.jsonl`
- `.jules/runs/specsmith-analysis-1/result.json`
- `.jules/runs/specsmith-analysis-1/pr_body.md`

## 🔜 Follow-ups
None.
