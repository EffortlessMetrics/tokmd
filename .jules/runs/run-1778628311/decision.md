## Options Considered

### Option A: Expand BDD scenario tests for missing analysis presets (Health, Deep) in `analysis_deep_w64.rs`
- **What it is**: The current BDD scenario section in `analysis_deep_w64.rs` only tests the `Receipt` preset (`bdd_receipt_counts_match_input`, `bdd_empty_repo_valid_receipt`, `bdd_modules_in_breakdown`). We can add equivalent scenarios for `Health` (to verify TODO extraction and complexity mapping) and `Deep` (to verify all enrichers process an empty or standard export safely).
- **Why it fits**: The prompt asks for "missing BDD/integration coverage for an important path" or "edge-case regression not locked in by tests". The top-level integration tests for `tokmd_analysis::analyze` currently miss explicit BDD behavioral statements for non-Receipt presets.
- **Trade-offs**: Adds integration test proof but no product code changes. Fits the "proof-improvement patch" allowance perfectly without polluting the actual application code with unnecessary changes.

### Option B: Fix feature gates silent skipping
- **What it is**: Modify `feature_gates_w71.rs` to enforce stricter gate handling.
- **Why it fits**: Feature gates are an edge case.
- **Trade-offs**: Might stray into unrelated testing logic or cross into standard unit testing instead of BDD/scenario testing.

## Decision
**Option A**. It perfectly aligns with the Specsmith persona (BDD/integration coverage) and explicitly provides a proof-improvement patch.
