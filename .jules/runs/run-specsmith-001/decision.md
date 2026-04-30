# Decision

The task calls for improving BDD/integration coverage or edge-case polish in the analysis-stack. I noticed `bdd_analyze_scenarios_w50.rs` in `crates/tokmd/tests/` uses the term "w50" and tests core analysis scenarios like JSON outputs, schema version presence, and effort reports.

However, `tokmd/tests/bdd_diff_scenarios_w50.rs` and `tokmd/tests/bdd_export_scenarios_w50.rs` and `tokmd/tests/bdd_lang_scenarios_w50.rs` and `bdd_module_scenarios_w50.rs` are present.
Wait, let's look at `crates/tokmd/tests/bdd_analyze_scenarios_w50.rs` - the file ends with a scenario `given_project_when_analyze_estimate_then_effort_model_present` testing the "estimate" preset.
I can add more BDD scenarios to `bdd_analyze_scenarios_w50.rs` to cover things like the `risk`, `supply`, and `health` presets, and feature flags.

Wait, looking at `crates/tokmd-analysis/tests/deep_analysis_w48.rs` vs `crates/tokmd-analysis/tests/analysis_depth_w62.rs` - these cover analysis logic deep tests.

Let's look at missing BDD scenarios in `bdd_analyze_scenarios_w50.rs`:
- Scenario testing `supply` preset: dependencies should be present.
- Scenario testing `risk` preset: git/complexity should be present.
- Scenario testing `health` preset: todo/complexity should be present.

Option A: Add BDD scenario tests for the missing analysis presets (`supply`, `risk`, `health`) in `crates/tokmd/tests/bdd_analyze_scenarios_w50.rs`. This directly fulfills the specsmith target "missing BDD/integration coverage for an important path".

Option B: Add tests for missing edge-cases around Git integration errors in analysis.

I'll choose Option A. It's safe, straight-forward, and provides direct BDD coverage for user-facing presets.
