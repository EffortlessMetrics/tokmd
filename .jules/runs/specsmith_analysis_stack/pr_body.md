## 💡 Summary
Added a BDD-style scenario test to `tokmd` testing the `analyze --preset health` command. The test locks in the expected output behavior, specifically ensuring `complexity` and `derived.todo` metrics are present in the JSON receipt.

## 🎯 Why
The test suite lacked a clear, scenario-driven test for the `health` preset in `bdd_analyze_scenarios_w50.rs`. Adding this test ensures edge-case polish around analysis behavior and prevents regressions by explicitly asserting that the `health` preset returns the expected code health insights (complexity and TODO metrics).

## 🔎 Evidence
- **File path:** `crates/tokmd/tests/bdd_analyze_scenarios_w50.rs`
- **Observed behavior:** Running the analysis with the `health` preset successfully populated the complexity and TODO metrics, but there was no BDD test enforcing this contract.
- **Receipt:**
  ```text
  $ cargo run -p tokmd -- analyze . --preset health --format json > health_output.json
  $ jq '.derived.todo' health_output.json
  {
    "total": 1182,
    "density_per_kloc": 3.16,
    "tags": [...]
  }
  $ cargo test -p tokmd --test bdd_analyze_scenarios_w50
  test given_project_when_analyze_health_then_todo_and_complexity_present ... ok
  ```

## 🧭 Options considered
### Option A (recommended)
- **What it is:** Add a BDD scenario for the `health` preset in `crates/tokmd/tests/bdd_analyze_scenarios_w50.rs`.
- **Why it fits:** It aligns perfectly with the Specsmith persona, adding behavior-level regression coverage for an important user path (code health analysis).
- **Trade-offs:** High value for minimal velocity cost, significantly improving test suite governance.

### Option B
- **What it is:** Refactor granular unit tests in `tokmd-analysis`.
- **When to choose it instead:** If there were no high-level missing integration tests and the unit tests were fundamentally broken.
- **Trade-offs:** Violates the anti-drift rule "Do not become a generic test cleanup lane."

## ✅ Decision
Option A was chosen to add behavior-driven test coverage for the `health` preset.

## 🧱 Changes made (SRP)
- `crates/tokmd/tests/bdd_analyze_scenarios_w50.rs`: Added `given_project_when_analyze_health_then_todo_and_complexity_present` scenario.

## 🧪 Verification receipts
```text
{"command": "cargo run -p tokmd -- analyze . --preset health --format json > health_output.json && jq keys health_output.json"}
{"command": "jq '.complexity' health_output.json | head -n 30"}
{"command": "jq '.derived.todo' health_output.json"}
{"command": "cargo test -p tokmd --test bdd_analyze_scenarios_w50"}
```

## 🧭 Telemetry
- **Change shape:** proof-improvement
- **Blast radius:** tests (modifies `bdd_analyze_scenarios_w50.rs` only)
- **Risk class:** Low. Only touches test files, zero risk to production code.
- **Rollback:** Safe to revert test file.
- **Gates run:** `cargo test -p tokmd --test bdd_analyze_scenarios_w50`, `cargo build --verbose`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`.

## 🗂️ .jules artifacts
- `.jules/runs/specsmith_analysis_stack/envelope.json`
- `.jules/runs/specsmith_analysis_stack/decision.md`
- `.jules/runs/specsmith_analysis_stack/receipts.jsonl`
- `.jules/runs/specsmith_analysis_stack/result.json`
- `.jules/runs/specsmith_analysis_stack/pr_body.md`

## 🔜 Follow-ups
None.
