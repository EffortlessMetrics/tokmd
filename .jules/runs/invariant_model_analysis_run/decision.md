## Option A
Add property-based tests for `build_confidence` in `crates/tokmd-analysis-effort/tests/proptest_confidence.rs`.
The `build_confidence` function computes an `EffortConfidence` score based on missing signals, generated file ratios, vendored file ratios, and presence of a delta. We can encode invariants such as:
1. `score` is always bounded `0.0 <= score <= 1.0`
2. Missing reports (git, complexity, api, dup) strictly decrease or keep the score equal to what it would be if they were present.
3. Increasing `generated_pct` and `vendored_pct` strictly decrease or keep the score equal.

## Option B
Add property tests for `classify_blast` and blast-radius clamp logic in `delta.rs`.
However, testing delta.rs involves `tokmd_git` integration for changed lines. It's much harder to prop-test cleanly without mocking `ExportData` and `GitReport` deeply.

## Decision
**Option A**. Testing the `confidence.rs` heuristics matches the "prover" / "property" criteria perfectly. It solidifies the bounds and monotonic properties of the confidence score, which is a key part of the effort model. I will add a new test file `crates/tokmd-analysis-effort/tests/proptest_confidence.rs` to enforce these invariants.
