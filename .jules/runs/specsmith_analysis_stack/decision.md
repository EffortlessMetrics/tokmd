# Specsmith Decision

## Option A (recommended)
Fix `ComplexityBaseline::from_analysis` to always extract `total_code_lines` and `total_files` from `receipt.derived`, even when the optional `receipt.complexity` feature is missing (e.g. on `--no-default-features` builds or when complexity analysis is toggled off).
- **Why it fits**: Directly addresses the regression risk where complexity being toggled off loses basic structural counts for the baseline contract. This fits Specsmith's mission to improve scenario coverage and regression coverage.
- **Trade-offs**:
  - **Structure**: Increases robustness by decoupling basic metric extraction from optional feature dependencies.
  - **Velocity**: A fast targeted fix that immediately protects the baseline contract.
  - **Governance**: Minimal surface area change, highly compliant with baseline semantics.

## Option B
Instead of changing `ComplexityBaseline::from_analysis`, change the CLI and scan engine to always include a stub `ComplexityReport` populated with just the `derived` metrics when complexity analysis fails or is turned off.
- **When to choose it**: If we wanted the schema to strictly enforce that all baselines originate from a formally completed complexity scan report.
- **Trade-offs**: Unnecessarily couples two independent systems in the core logic. Requires much deeper refactoring across `tokmd-scan` and `tokmd-types` just to satisfy `tokmd-analysis-types`.

## Decision
**Option A**. It's the most correct, targeted structural solution that ensures baseline logic properly respects gracefully degraded `AnalysisReceipt` structures. We've written a concrete test case mimicking the degraded receipt to guarantee it behaves properly.
