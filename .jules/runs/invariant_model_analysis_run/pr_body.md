## 💡 Summary
Added property tests for the effort confidence scoring engine. These tests assert invariants such as confidence bounds, score mapping correctly to confidence levels, and the monotonicity of adding missing context signals.

## 🎯 Why
The `build_confidence` function uses heuristics combined with various input analyses (complexity, git history, duplication, etc.) to derive a confidence score. Locking these behaviors with property tests prevents regressions where an absent signal incorrectly implies a higher confidence or boundary conditions slip out of the `0.0..=1.0` range.

## 🔎 Evidence
Tested `build_confidence` using generated input reports covering `EffortSizeBasis`, `DerivedReport`, `GitReport`, `ComplexityReport`, `ApiSurfaceReport`, and `DuplicateReport`.
- Bounding invariant ensures `score >= 0.0 && score <= 1.0` and aligns correctly with `EffortConfidenceLevel`.
- Monotonicity invariant ensures `score_all >= score_none` when optional context signals are added.

## 🧭 Options considered
### Option A (recommended)
- Test `build_confidence` inside `crates/tokmd-analysis-effort/tests/proptest_confidence.rs`.
- Fits the model logic cleanly and exercises multiple boundary combinations without requiring integration testing.
- Trade-offs: Minor increase in test duration due to proptest iterations, but strongly locks in correct logic.

### Option B
- Focus on testing the delta calculation engine (`classify_blast`).
- Trade-offs: Delta calculations require heavier integration with git metrics and export data matching, making generic randomized proptesting brittle and overly mocked.

## ✅ Decision
Option A was chosen. Adding specific monotonic checks for `build_confidence` effectively covers the largest source of fuzzy logic heuristics in the effort engine.

## 🧱 Changes made (SRP)
- Added `crates/tokmd-analysis-effort/tests/proptest_confidence.rs` to house the new `build_confidence` property tests.

## 🧪 Verification receipts
```text
{"command": "cargo test -p tokmd-analysis-effort", "outcome": "ok", "notes": "6 tests passed for new property tests"}
{"command": "cargo clippy -- -D warnings", "outcome": "ok", "notes": "clippy pass"}
{"command": "cargo fmt -- --check", "outcome": "ok", "notes": "fmt pass"}
```

## 🧭 Telemetry
- Change shape: New test file added.
- Blast radius: Zero API, schema, or production logic impact. Purely an improvement to tests.
- Risk class: Low - Test-only patch.
- Rollback: Revert the added test file.
- Gates run: `cargo test`, `cargo fmt`, `cargo clippy`.

## 🗂️ .jules artifacts
- `.jules/runs/invariant_model_analysis_run/envelope.json`
- `.jules/runs/invariant_model_analysis_run/decision.md`
- `.jules/runs/invariant_model_analysis_run/receipts.jsonl`
- `.jules/runs/invariant_model_analysis_run/result.json`
- `.jules/runs/invariant_model_analysis_run/pr_body.md`

## 🔜 Follow-ups
None immediately.
