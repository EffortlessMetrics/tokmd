## 💡 Summary
Added an explicit property test using `proptest` to structurally prove serialization and deserialization invariants for `DeterminismBaseline`.

## 🎯 Why
The `DeterminismBaseline` type plays an important role in proving execution behavior, but lacked local inline property testing. The testing that did exist was limited to hardcoded example inputs in integration suites. This change moves us closer towards comprehensive, parameterised invariant testing by ensuring the model structure can survive `serde` translation robustly under a wide gamut of generated inputs.

## 🔎 Evidence
Minimal proof:
- file path: `crates/tokmd-analysis-types/src/baseline/determinism.rs`
- observed behavior / finding: No inline proptests for the structure existed despite adjacent baseline structures having them.
- command receipt demonstrating it: `cargo test -p tokmd-analysis-types --lib baseline::determinism` executes the new deterministic test successfully over arbitrarily generated configurations.

## 🧭 Options considered
### Option A (recommended)
- what it is: Add a `tests` module inside `crates/tokmd-analysis-types/src/baseline/determinism.rs` that imports `proptest` and verifies `DeterminismBaseline` correctly round-trips via `serde_json` for a variety of pseudo-random configurations.
- why it fits this repo and shard: Co-locating structural invariances closely aligns with how models are structured across analysis surfaces in the workspace.
- trade-offs: Structure / Velocity / Governance: Highly targeted proof-patch improving local correctness guarantees (Velocity + Structure + Governance positive).

### Option B
- what it is: Rely on integration level tests inside `tests/analysis_types_depth_w61.rs`.
- when to choose it instead: If the constraints were dependent on orchestration outside the DTO boundaries.
- trade-offs: Less locally visible; integration tests shouldn't handle what unit structural property tests can.

## ✅ Decision
Selected **Option A** to guarantee structure roundtripping works across generated fuzzable ranges via standard `proptest!` macro.

## 🧱 Changes made (SRP)
- `crates/tokmd-analysis-types/src/baseline/determinism.rs`: Included inline `tests` module testing serde.

## 🧪 Verification receipts
```text
{"cmd": "cargo test -p tokmd-analysis-types --lib baseline::determinism", "summary": "Tested inline proptest for DeterminismBaseline", "status": "success"}
{"cmd": "cargo fmt -- --check", "summary": "Formatting check", "status": "success"}
{"cmd": "cargo clippy -- -D warnings", "summary": "Clippy check", "status": "success"}
```

## 🧭 Telemetry
- Change shape: Added tests.
- Blast radius (API / IO / docs / schema / concurrency / compatibility / dependencies): Testing surface exclusively.
- Risk class + why: Lowest risk. No production behaviour changed.
- Rollback: `git checkout crates/tokmd-analysis-types/src/baseline/determinism.rs`
- Gates run: `cargo test`, `cargo fmt`, `cargo clippy`.

## 🗂️ .jules artifacts
- `.jules/runs/invariant_model_analysis/envelope.json`
- `.jules/runs/invariant_model_analysis/decision.md`
- `.jules/runs/invariant_model_analysis/receipts.jsonl`
- `.jules/runs/invariant_model_analysis/result.json`
- `.jules/runs/invariant_model_analysis/pr_body.md`

## 🔜 Follow-ups
None.
