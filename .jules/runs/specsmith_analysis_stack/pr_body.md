## 💡 Summary
Ensure structural metrics (`total_files`, `total_code_lines`) are correctly extracted in `ComplexityBaseline::from_analysis` even when the `complexity` feature is missing from the receipt.

## 🎯 Why
When aggregating metrics in `tokmd-analysis-types` (e.g., `ComplexityBaseline::from_analysis`), fallback logic must extract structural counts from `receipt.derived` if optional feature-specific reports (like `receipt.complexity`) are `None` due to `--no-default-features` builds or scan toggles. Previously, these were zeroed out, masking the correct baseline totals.

## 🔎 Evidence
- **File**: `crates/tokmd-analysis-types/src/lib.rs`
- **Finding**: The variables `total_code_lines` and `total_files` were nested inside the `if let Some(ref complexity_report) = receipt.complexity` block.
- **Proof**: Created `crates/tokmd-analysis-types/tests/baseline_fallback.rs` with `receipt.complexity = None` and validated that metrics are properly extracted. Ran `cargo test -p tokmd-analysis-types`.

## 🧭 Options considered
### Option A (recommended)
- Extracted `total_code_lines` and `total_files` outside the complexity feature check so they can be captured unconditionally.
- **Why it fits**: Directly addresses the regression risk where complexity being toggled off loses basic structural counts for the baseline contract. Fits Specsmith's mission to improve scenario coverage and regression coverage.
- **Trade-offs**: Increases robustness without deep structural refactoring; highly compliant with baseline semantics.

### Option B
- Change the CLI and scan engine to always include a stub `ComplexityReport` populated with just the `derived` metrics when complexity analysis fails or is turned off.
- **When to choose it**: If we wanted the schema to strictly enforce that all baselines originate from a formally completed complexity scan report.
- **Trade-offs**: Unnecessarily couples two independent systems in the core logic. Requires much deeper refactoring across `tokmd-scan` and `tokmd-types`.

## ✅ Decision
**Option A**. It's the most correct, targeted structural solution that ensures baseline logic properly respects gracefully degraded `AnalysisReceipt` structures. We've written a concrete test case mimicking the degraded receipt to guarantee it behaves properly.

## 🧱 Changes made (SRP)
- `crates/tokmd-analysis-types/src/lib.rs`
- `crates/tokmd-analysis-types/tests/baseline_fallback.rs`

## 🧪 Verification receipts
```text
cargo test -p tokmd-analysis-types
cargo fmt -- --check
cargo clippy -p tokmd-analysis-types -- -D warnings
```

## 🧭 Telemetry
- **Change shape**: Feature fix + tests
- **Blast radius**: Low. Restricted to `tokmd-analysis-types` metric aggregation.
- **Risk class**: Low, logic only executes under edge-case conditions (missing complexity metrics).
- **Rollback**: Trivial revert.
- **Gates run**: `cargo test -p tokmd-analysis-types`, `cargo fmt`, `cargo clippy`.

## 🗂️ .jules artifacts
- `.jules/runs/specsmith_analysis_stack/envelope.json`
- `.jules/runs/specsmith_analysis_stack/decision.md`
- `.jules/runs/specsmith_analysis_stack/receipts.jsonl`
- `.jules/runs/specsmith_analysis_stack/result.json`
- `.jules/runs/specsmith_analysis_stack/pr_body.md`

## 🔜 Follow-ups
None.
