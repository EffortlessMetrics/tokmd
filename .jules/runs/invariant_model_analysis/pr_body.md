## 💡 Summary
Tightened mathematical invariants for `compute_maintainability_index`.

## 🎯 Why
The SEI maintainability index has sharp boundary conditions, especially around Halstead Volume fallbacks. Adding exact boundary and mathematical proofs using `proptest` ensures these models remain perfectly coherent with expectations under all potential variable domains.

## 🔎 Evidence
File path: `crates/tokmd-analysis/src/maintainability/tests/properties.rs`.
The proptest suite previously only tested weak inequalities for variables like CC decreasing scores, but lacked explicit assertions regarding exact calculations (like bounding the specific `-5.2 * V.ln()` penalty relative to the simplified version).

## 🧭 Options considered
### Option A (recommended)
- Tighten proptests for `compute_maintainability_index` in `crates/tokmd-analysis/src/maintainability/tests/properties.rs`.
- Why it fits: Strengthens invariant coverage of core analysis models.
- Trade-offs: Adds a slight amount of runtime for properties validation but explicitly locks in correctness.

### Option B
- Add a new `properties.rs` file for other analysis models, like `license` reporting.
- When to choose: If maintainability was already sufficiently mapped out mathematically.
- Trade-offs: Does not tighten the known mathematical model of the maintainability index bounds.

## ✅ Decision
Chose Option A to add explicit exact formula invariants and strict monotonicity constraints via `proptest`.

## 🧱 Changes made (SRP)
- `crates/tokmd-analysis/src/maintainability/tests/properties.rs`

## 🧪 Verification receipts
```text
cargo test -p tokmd-analysis --lib properties
cargo clippy -p tokmd-analysis -- -D warnings
```

## 🧭 Telemetry
- Change shape: Invariant Proofs
- Blast radius: Internal test surface only
- Risk class: Low
- Rollback: `git checkout crates/tokmd-analysis/src/maintainability/tests/properties.rs`
- Gates run: targeted cargo test, clippy

## 🗂️ .jules artifacts
- `.jules/runs/invariant_model_analysis/envelope.json`
- `.jules/runs/invariant_model_analysis/decision.md`
- `.jules/runs/invariant_model_analysis/receipts.jsonl`
- `.jules/runs/invariant_model_analysis/result.json`
- `.jules/runs/invariant_model_analysis/pr_body.md`

## 🔜 Follow-ups
None.
