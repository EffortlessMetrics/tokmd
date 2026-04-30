## 💡 Summary
Refactored inline sorting logic for data rows (`LangRow`, `ModuleRow`, `FileRow`) into exposed public functions in `tokmd-model` (`sort_lang_rows`, `sort_module_rows`, `sort_file_rows`). Updated integration and determinism tests in `tokmd-model` and `tokmd-types` to use these central functions rather than redefining duplicate sorting closures.

## 🎯 Why
This addresses a memory gap to ensure integration tests test the actual sorting determinism used in production. It centralizes sorting logic and prevents drift or inconsistencies in tests testing row structures.

## 🔎 Evidence
- Found multiple inline closures across `crates/tokmd-model/tests` and `crates/tokmd-types/tests` testing determinism by sorting rows with `b.code.cmp(&a.code).then_with(...)`.
- `cargo test -p tokmd --test determinism_regression` passes correctly showing we didn't break deterministic sorting contracts.

## 🧭 Options considered
### Option A (recommended)
- Expose `sort_lang_rows`, `sort_module_rows`, and `sort_file_rows` as public functions in `tokmd-model` and refactor tests to use them.
- **Why it fits**: Directly satisfies the instruction to test sorting determinism using exposed public functions rather than redefining duplicate sorting logic in tests. Resolves a friction point.
- **Trade-offs**: Adds 3 new utility functions to the public API of `tokmd-model`.

### Option B
- Add a macro or test utility function solely within the tests directory.
- **When to choose it instead**: If exposing these sorting functions to the public API is deemed a stability risk.
- **Trade-offs**: Doesn't truly test the exact determinism logic used by the library.

## ✅ Decision
Option A was chosen as it strictly follows the Gatekeeper instructions to standardize determinism checks via exported standalone functions.

## 🧱 Changes made (SRP)
- `crates/tokmd-model/src/lib.rs`
- `crates/tokmd-model/tests/deep.rs`
- `crates/tokmd-model/tests/deep_model_w49.rs`
- `crates/tokmd-model/tests/determinism_w66.rs`
- `crates/tokmd-model/tests/mutation_coverage_w50.rs`
- `crates/tokmd-model/tests/properties.rs`
- `crates/tokmd-model/tests/proptest_w42.rs`
- `crates/tokmd-model/tests/proptest_w72.rs`
- `crates/tokmd-types/tests/determinism_props.rs`
- `crates/tokmd-types/tests/determinism_proptest.rs`
- `crates/tokmd-types/tests/mutation_coverage_w50.rs`

## 🧪 Verification receipts
```text
cargo test -p tokmd-model (pass)
cargo test -p tokmd-types (pass)
cargo test -p tokmd --test determinism_regression (pass)
```

## 🧭 Telemetry
- Change shape: Refactoring
- Blast radius: Internal tests, API addition. Low risk as core determinism logic is untouched, just wrapped.
- Risk class: Low
- Rollback: Revert the PR
- Gates run: `cargo test`

## 🗂️ .jules artifacts
- `.jules/runs/gatekeeper_determinism/envelope.json`
- `.jules/runs/gatekeeper_determinism/decision.md`
- `.jules/runs/gatekeeper_determinism/receipts.jsonl`
- `.jules/runs/gatekeeper_determinism/result.json`
- `.jules/runs/gatekeeper_determinism/pr_body.md`

## 🔜 Follow-ups
None
