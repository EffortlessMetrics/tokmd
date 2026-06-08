## 💡 Summary
Added two missing property-based invariants to the `tokmd-analysis` entropy module. This tightens verification around entropy scan determinism and ensures suspect paths are strictly bounded by input lists.

## 🎯 Why
The `build_entropy_report` function returns suspects based on computed entropy, but there were no invariant checks guaranteeing deterministic behavior or strict adherence to the input file list. Adding these reduces edge-case risk.

## 🔎 Evidence
Added tests in `crates/tokmd-analysis/src/entropy/tests/properties.rs`:
- `deterministic_entropy_report`
- `suspect_paths_must_be_in_inputs`

## 🧭 Options considered
### Option A (recommended)
- Add missing properties to `entropy/tests/properties.rs`.
- Why it fits: Directly aligns with the `property` gate expectations and `Prover` style by adding clear deterministic proof constraints.
- Trade-offs: Increases test execution time marginally, but adds significant confidence.

### Option B
- Refactor the entropy engine to structurally prevent non-determinism.
- When to choose: If the engine is fundamentally unstable. Currently, adding tests is sufficient to prove stability.
- Trade-offs: Unnecessary risk and churn.

## ✅ Decision
Implemented Option A.

## 🧱 Changes made (SRP)
- `crates/tokmd-analysis/src/entropy/tests/properties.rs`

## 🧪 Verification receipts
```text
cargo test --verbose
cargo build --verbose
cargo fmt -- --check
cargo clippy -- -D warnings
```

## 🧭 Telemetry
- Change shape: Additions to property tests.
- Blast radius: Test-only impact.
- Risk class: Low
- Rollback: git checkout crates/tokmd-analysis/src/entropy/tests/properties.rs
- Gates run: `cargo build --verbose`, `CI=true cargo test --verbose`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`

## 🗂️ .jules artifacts
- `.jules/runs/run-invariant-001/envelope.json`
- `.jules/runs/run-invariant-001/decision.md`
- `.jules/runs/run-invariant-001/receipts.jsonl`
- `.jules/runs/run-invariant-001/result.json`
- `.jules/runs/run-invariant-001/pr_body.md`

## 🔜 Follow-ups
None.
