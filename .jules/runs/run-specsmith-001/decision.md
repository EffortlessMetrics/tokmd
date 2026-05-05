# Decision

The task calls for improving BDD/integration coverage or edge-case polish in the analysis-stack. I noticed `bdd_analyze_scenarios_w50.rs` in `crates/tokmd/tests/` uses the term "w50" and tests core analysis scenarios like JSON outputs, schema version presence, and effort reports.

Option A: Add BDD scenario tests for the missing analysis presets (`supply`, `risk`, `health`) in `crates/tokmd/tests/bdd_analyze_scenarios_w50.rs`.
Option B: Abandon the redundant BDD test patch since it was superseded by #1578, and record a Learning PR covering this workflow friction edge case.

I chose Option A initially, but it was superseded during review. I am pivoting to Option B to produce a Learning PR.
