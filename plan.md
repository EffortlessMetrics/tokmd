1. **Analyze tokmd-analysis orchestration**
   - Review how `tokmd-analysis` manages derived metrics, presets, limits, and optional features for analysis orchestration.

2. **Add Missing BDD Coverage to tokmd-analysis**
   - Create `crates/tokmd-analysis/tests/bdd.rs` to add BDD-style scenario tests.
   - Scenario 1: Add a test `scenario_multi_language_polyglot_and_distribution` to verify that when analyzing a multi-module export with different languages, polyglot and distribution calculations correctly aggregate and rank languages.
   - Scenario 2: Add a test `scenario_analysis_limits_guardrails` to ensure that artificial analysis limits constrain operations correctly while derived totals still capture the whole input.
   - Scenario 3: Add a test `scenario_context_window_fitting` to verify context window bound tracking.
   - Scenario 4: Add a test `scenario_missing_enrichers_for_disabled_features` to verify that attempting a deep preset when features are gated gracefully returns `ScanStatus::Partial` with warnings.

3. **Verify Tests**
   - Run `cargo test -p tokmd-analysis --test bdd` to verify that the scenarios are properly handled.

4. **Complete pre-commit steps**
   - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.

5. **Submit Changes**
   - Commit the changes and submit the patch.
