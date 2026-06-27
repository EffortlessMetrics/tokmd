# Decision

## Option A: Improve BDD coverage in `tokmd-gate` for complex missing policy conditions
The `tokmd-gate` crate already has a `bdd.rs` file which covers some basic budget and ratchet scenarios. We can improve scenario coverage by adding BDD tests for policies that include `allow_missing_baseline` and `allow_missing_current`, combined with the edge cases of missing properties under these conditions.

## Option B: Add a dedicated `bdd.rs` file in `tokmd-analysis`
The `tokmd-analysis` crate contains deep and orchestration tests, and its `analysis_deep_w64.rs` test file has a few BDD tests (like `bdd_receipt_counts_match_input`), but lacks a dedicated `bdd.rs` file for scenario coverage. Adding `crates/tokmd-analysis/tests/bdd.rs` to cover full end-to-end BDD scenarios for analysis presets (e.g. "Given a multi-language repo, When running deep analysis, Then it correctly calculates cross-module polyglot, entropy and effort reports") would provide much-needed scenario coverage.

**Decision: Option B.**
We will create `crates/tokmd-analysis/tests/bdd.rs` to add integration-level scenario coverage for `tokmd-analysis` behaviors, particularly focusing on multi-module and complex scenarios, which aligns with Specsmith's mission to "Improve scenario coverage, regression coverage, and edge-case polish."
