## 💡 Summary
Improved the BDD scenario test for the `health` analysis preset to actually prove TODO tags are parsed correctly. Previously the test just checked that a `todo` JSON object existed, which hid the fact that no tags were actually found in the test fixture.

## 🎯 Why
The `given_project_when_analyze_health_then_todo_and_complexity_present` test asserted that `json["derived"].get("todo").is_some()`. This passes even when the `todo` counts are zero (which they were, because `crates/tokmd/tests/data` contains no TODOs). This missing test coverage for an important feature means we weren't truly locking in the behavior of the tag parser in the CLI pipeline. We needed a test that explicitly proves TODOs are counted.

## 🔎 Evidence
Before the change, running the CLI on the test data directory showed zero tags:
```text
$ cargo run --bin tokmd -- analyze crates/tokmd/tests/data/src/ --preset health --format json | jq '.derived.todo'
{
  "total": 0,
  "density_per_kloc": 0.0,
  "tags": [
    {
      "tag": "FIXME",
      "count": 0
    },
    ...
  ]
}
```

## 🧭 Options considered
### Option A (recommended)
- **What it is**: Inject real TODO tags into a temporary file and verify they are counted correctly in the JSON output.
- **Why it fits this repo and shard**: Target ranking #1 is "missing BDD/integration coverage for an important path" and #3 is "confusing scenario setup that hides real behavior". The `interfaces` shard explicitly covers `crates/tokmd/tests/**`.
- **Trade-offs**: Improves fidelity of the test suite with minimal blast radius. Structure + Velocity + Governance win.

### Option B
- **What it is**: Add a BDD scenario for the `--children` flag edge cases.
- **When to choose it instead**: If the TODO test was already comprehensive or if we found a bug in the children flag implementation.
- **Trade-offs**: We already have tests covering the children flag, so coverage there is decent. The missing TODO coverage is a clear gap.

## ✅ Decision
Option A. Upgraded the test to write a temporary file with actual TODO, FIXME, and HACK tags before running the CLI, and then asserts that the counts in the JSON output match the injected comments. This prevents regressions in tag parsing logic and aligns the test fixture with the test's semantic intent.

## 🧱 Changes made (SRP)
- `crates/tokmd/tests/bdd_analyze_scenarios_w50.rs`:
  - Updated `given_project_when_analyze_health_then_todo_and_complexity_present` to use a `tempdir` with a file containing TODO tags.
  - Added specific assertions on the tag counts instead of `.is_some()`.

## 🧪 Verification receipts
```text
$ CI=true cargo test -p tokmd --test bdd_analyze_scenarios_w50 given_project_when_analyze_health_then_todo_and_complexity_present -- --nocapture
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.03s

$ CI=true cargo test -p tokmd --test bdd_analyze_scenarios_w50
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
```

## 🧭 Telemetry
- Change shape: Test enhancement
- Blast radius: Tests only.
- Risk class: Very low. It only changes test assertions.
- Rollback: `git checkout crates/tokmd/tests/bdd_analyze_scenarios_w50.rs`
- Gates run: `cargo build --verbose`, `CI=true cargo test --verbose`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`

## 🗂️ .jules artifacts
- `.jules/runs/specsmith-interfaces-01/envelope.json`
- `.jules/runs/specsmith-interfaces-01/decision.md`
- `.jules/runs/specsmith-interfaces-01/receipts.jsonl`
- `.jules/runs/specsmith-interfaces-01/result.json`
- `.jules/runs/specsmith-interfaces-01/pr_body.md`

## 🔜 Follow-ups
None.
