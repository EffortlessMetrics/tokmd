# Decision: Specsmith Interfaces

## Investigation

I investigated the `interfaces` shard (crates/tokmd, crates/tokmd-core, crates/tokmd-config). The goal is to improve scenario coverage, regression coverage, or edge-case polish in `tokmd`.

While checking the `analyze` CLI tool's BDD integration tests in `crates/tokmd/tests/bdd_analyze_scenarios_w50.rs`, I noticed it exercises `receipt`, `health`, `estimate`, and `fun` presets. However, it entirely misses the `risk` preset, which is an important path that calculates Git-backed hotspots and risk metrics in addition to health metrics.

A quick scratchpad test proved that running `tokmd analyze . --preset risk --format json` on the fixture root succeeds and correctly outputs the `git` telemetry section alongside `complexity` and `derived.todo`. Without a BDD test locking this in, changes to the `risk` preset's CLI integration (like accidentally breaking git resolution) wouldn't be caught by the core BDD analyze scenario suite.

Option A: Add a BDD integration test for the `risk` preset to `bdd_analyze_scenarios_w50.rs`. This directly satisfies the Specsmith mission (missing BDD/integration coverage for an important path) and locks in the interface behavior.

Option B: Create a learning PR noting the gap but not fixing it.

I will choose Option A because it clearly fulfills the criteria for a proof-improvement patch.
