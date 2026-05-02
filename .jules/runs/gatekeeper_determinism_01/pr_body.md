## 💡 Summary
Extracted inline deterministic sorting logic for `LangRow`, `ModuleRow`, and `FileRow` into public functions. Refactored all property and integration tests to use these shared invariants.

## 🎯 Why
Tests were manually redefining complex `sort_by` closures (e.g., descending by code, then ascending by name) to assert on determinism. This created a fragile proof surface where a change in production sorting would not naturally fail tests unless the duplicated test closures were also changed.

## 🔎 Evidence
- Dozens of inline instances of `rows.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.lang.cmp(&b.lang)));` spread across `crates/tokmd-model/tests/`.
- Proof of determinism requires tests and production to use the identical function.

## 🧭 Options considered
### Option A (recommended)
- Extract standalone `sort_lang_rows`, `sort_module_rows`, and `sort_file_rows` functions.
- Why it fits: Directly eliminates duplicate closures and tightens contract invariants.
- Trade-offs: Structure is improved, velocity is high, governance guarantees determinism matching.

### Option B
- Add large determinism snapshot files instead.
- When to choose: When we can't extract sorting logic easily.
- Trade-offs: Fails to address the underlying duplicated closures.

## ✅ Decision
Option A. Centralizing the sorting logic ensures tests and application code never drift.

## 🧱 Changes made (SRP)
- `crates/tokmd-model/src/lib.rs`
- `crates/tokmd-model/tests/deep_model_w49.rs`
- `crates/tokmd-model/tests/proptest_deep.rs`
- `crates/tokmd-model/tests/deep.rs`
- `crates/tokmd-model/tests/determinism_w66.rs`
- `crates/tokmd-model/tests/proptest_w72.rs`
- `crates/tokmd-model/tests/proptest_w42.rs`
- `crates/tokmd-model/tests/properties.rs`
- `crates/tokmd-model/tests/mutation_coverage_w50.rs`

## 🧪 Verification receipts
```text
cargo test -p tokmd-model
test result: ok. 52 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s
cargo xtask version-consistency
✓ Cargo crate versions match 1.10.0.
cargo xtask publish --plan
cargo clippy -- -D warnings
```

## 🧭 Telemetry
- Change shape: Core model refactoring
- Blast radius: Low risk; logic is exact replacement.
- Risk class: Low. Preserves exactly the same `sort_by` logic.
- Rollback: `git reset --hard`
- Gates run: `cargo test -p tokmd-model`, `cargo fmt`, `cargo clippy`, `cargo xtask version-consistency`, `cargo xtask publish --plan`

## 🗂️ .jules artifacts
- `.jules/runs/gatekeeper_determinism_01/envelope.json`
- `.jules/runs/gatekeeper_determinism_01/decision.md`
- `.jules/runs/gatekeeper_determinism_01/receipts.jsonl`
- `.jules/runs/gatekeeper_determinism_01/result.json`
- `.jules/runs/gatekeeper_determinism_01/pr_body.md`

## 🔜 Follow-ups
None.
