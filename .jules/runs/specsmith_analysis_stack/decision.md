# Decision

## Option A (recommended)
Add a BDD scenario for the `health` preset in `crates/tokmd/tests/bdd_analyze_scenarios_w50.rs`.
- **What it is:** Tests that `tokmd analyze --preset health` correctly outputs `complexity` and `derived.todo` metrics in JSON format.
- **Why it fits:** The Specsmith persona focuses on scenario coverage and regression coverage. The `health` preset is a major feature that aggregates TODO scanning and complexity analysis, and a BDD scenario explicitly locking down this behavior improves coverage without modifying unrelated files.
- **Trade-offs:** Minimal velocity cost, improves governance by locking in behavior, keeps the analysis test suite robust and regression-free.

## Option B
Refactor `crates/tokmd-analysis/src/content/tests/` to use more granular unit tests.
- **What it is:** Clean up generic assertions in the content parsing tests.
- **Why not:** The prompt explicitly warns: "Do not touch: generic assertion cleanup not tied to a scenario" and "Do not become a generic test cleanup lane."

## ✅ Decision
Option A. It adds a true behavior-driven scenario test for the `health` preset, providing meaningful regression coverage without violating the anti-drift rules.
