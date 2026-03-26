# PR Glass Cockpit

Make review boring. Make truth cheap.

## 💡 Summary
Verified commands from `docs/tutorial.md` and `README.md` by writing executable doctests to prevent silent drift.

## 🎯 Why / Threat model
Documentation frequently drifts from API realities. When users encounter failing commands from onboarding guides, trust is lost. Ensuring our examples compile and execute protects our UX invariants.

## 🔎 Finding (evidence)
- `tokmd tools` formatting examples in `tutorial.md`
- `tokmd analyze` preset examples in `tutorial.md`
- `tokmd badge` generation examples in `README.md`
These were entirely unverified and could break without CI noticing.

## 🧭 Options considered
### Option A (recommended)
- Add executable integration tests via `assert_cmd` within `crates/tokmd/tests/docs.rs`.
- Why it fits this repo: Maintains our "docs as tests" strategy and verifies the CLI contract exactly as users invoke it.
- Trade-offs: Increases test execution time marginally, but adds 100% confidence to onboarding flows.

### Option B
- Write a bash script that parses markdown and evaluates it.
- When to choose it instead: If tests span multiple repositories or involve heavy environment setup.
- Trade-offs: Flaky, custom parser required, hard to integrate with cargo's test harness.

## ✅ Decision
Option A. Rust integration tests provide structured assertions and integrate seamlessly with `cargo test`.

## 🧱 Changes made (SRP)
- Added `recipe_tools_export_schemas` test to `crates/tokmd/tests/docs.rs`
- Added `recipe_analyze_presets` test to `crates/tokmd/tests/docs.rs`
- Added `recipe_badge_generation` test to `crates/tokmd/tests/docs.rs`

## 🧪 Verification receipts
```
test recipe_badge_generation ... ok
test recipe_tools_export_schemas ... ok
test recipe_analyze_presets ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
```

## 🧭 Telemetry
- Change shape: New integration tests added
- Blast radius: Testing only. No production code modified.
- Risk class: None.
- Merge-confidence gates: `test`

## 🗂️ .jules updates
Added passing run to `.jules/docs/ledger.json`.
