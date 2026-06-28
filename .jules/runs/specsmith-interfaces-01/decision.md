# Decision

## 🧭 Options considered

### Option A: Fix the missing TODO tags coverage in health analysis (recommended)
- **What it is**: The `bdd_analyze_scenarios_w50.rs` file contains a test named `given_project_when_analyze_health_then_todo_and_complexity_present` which currently asserts that `json["derived"].get("todo").is_some()`. However, we've shown that `jq '.derived.todo'` on the test fixture returns an empty result `{"total": 0, "density_per_kloc": 0.0, "tags": [{"tag": "FIXME", "count": 0}, ...]}` because the fixture contains no actual TODO comments. The test passes only because the `todo` key is *present* (with default zero values). This hides the real behavior: whether TODO tags are actually parsed, counted, and surfaced correctly. We should add a real file with TODO tags to the test fixture to prove that the pipeline parses them, updates counts, and maps them to JSON output correctly.
- **Why it fits this repo and shard**: This perfectly fits the Specsmith persona, which states: "Improve scenario coverage, regression coverage, and edge-case polish." Target ranking #1 is "missing BDD/integration coverage for an important path" and #3 is "confusing scenario setup that hides real behavior". The `interfaces` shard explicitly covers `crates/tokmd/tests/**`.
- **Trade-offs**:
  - Structure: Improves the fidelity of the integration test suite.
  - Velocity: Quick to implement, minimal blast radius.
  - Governance: Low risk, purely a test coverage/proof improvement patch.

### Option B: Add a BDD scenario for the --children flag edge cases
- **What it is**: The `--children` flag operates differently between `module` / `export` and `lang` subcommands. We could add more scenario tests covering these nuances.
- **When to choose it instead**: If the TODO test was already comprehensive or if we found a bug in the children flag implementation.
- **Trade-offs**: We already have tests like `given_project_when_module_children_separate_then_mode_recorded` and `given_project_with_embedded_code_when_children_collapse_then_mode_recorded` passing, so coverage there is decent. The missing TODO coverage is a clear gap where the test name implies a behavior but the fixture doesn't support it.

## ✅ Decision
Option A. I will improve the `given_project_when_analyze_health_then_todo_and_complexity_present` test to write a temporary file with actual TODO, FIXME, and HACK tags before running the CLI, and then assert that the counts in the JSON output match the injected comments. This prevents regressions in tag parsing logic and aligns the test fixture with the test's semantic intent.
