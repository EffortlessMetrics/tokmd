## 💡 Summary
Added missing BDD-style scenario tests to `tokmd-analysis` to cover full end-to-end orchestration behavior.

## 🎯 Why
The `tokmd-analysis` crate coordinates derived metrics, presets, limits, and optional feature flags across the workspace. While we have deep tests for unit behaviors, we lacked integration-level BDD scenario tests that prove how everything interacts when multiple files, modules, and limits are combined.

## 🔎 Evidence
- **File**: `crates/tokmd-analysis/tests/bdd.rs`
- **Behavior**: Added BDD-style scenarios for:
  - Multi-language polyglot and distribution ranking.
  - Limit guardrails preserving derived totals.
  - Context window fitting behavior.
  - Graceful fallback to `ScanStatus::Partial` with warnings when optional features are gated.

## 🧭 Options considered
### Option A
- what it is: Improve BDD coverage in `tokmd-gate` for complex missing policy conditions.
- when to choose it instead: If the main risk factor was policy evaluation errors.
- trade-offs: We already have `tokmd-gate/tests/bdd.rs` but lacked `tokmd-analysis/tests/bdd.rs`.

### Option B (recommended)
- what it is: Add a dedicated `bdd.rs` file in `tokmd-analysis`.
- why it fits this repo and shard: It directly locks in the core orchestration flows of the `analysis-stack` shard, ensuring regression coverage for multi-module reports.
- trade-offs: Structure is improved by formalizing scenario tests.

## ✅ Decision
Option B. We created `crates/tokmd-analysis/tests/bdd.rs` to provide missing BDD-style integration coverage for `tokmd-analysis` behaviors, particularly around multi-module aggregation and limits.

## 🧱 Changes made (SRP)
- `crates/tokmd-analysis/tests/bdd.rs` (new file with BDD scenarios)

## 🧪 Verification receipts
```text
running 4 tests
test scenario_analysis_limits_guardrails ... ok
test scenario_context_window_fitting ... ok
test scenario_missing_enrichers_for_disabled_features ... ok
test scenario_multi_language_polyglot_and_distribution ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## 🧭 Telemetry
- Change shape: Test addition
- Blast radius: None (tests only)
- Risk class: Low
- Rollback: Revert the test file
- Gates run: `core-rust` (test, build, fmt, clippy)

## 🗂️ .jules artifacts
- `envelope.json`
- `decision.md`
- `result.json`
- `pr_body.md`

## 🔜 Follow-ups
None.
