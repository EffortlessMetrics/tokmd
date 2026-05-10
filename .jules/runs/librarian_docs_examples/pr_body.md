## 💡 Summary
Updated the `docs_schema_w72` and `docs_w43` xtask tests to look for the `BASELINE_VERSION` constant in its new location. The constant was recently moved from `crates/tokmd-analysis-types/src/lib.rs` to `baseline.rs`, which broke the string-matching test scanner.

## 🎯 Why
The `cargo test -p xtask` gate was failing because the test utilities (`read_const_u32` and `read_schema_constant`) parse Rust source files as raw text to find schema constants. When `BASELINE_VERSION` was moved out of `lib.rs` into the `baseline` module, the tests could no longer find it, breaking the factual drift guardrails that ensure our `SCHEMA.md` and `baseline.schema.json` stay in sync with the code.

## 🔎 Evidence
- `xtask/tests/docs_schema_w72.rs`
- `xtask/tests/docs_w43.rs`
- Running `cargo test -p xtask` resulted in:
  ```text
  thread 'schema_md_baseline_version_matches_source' panicked at xtask/tests/docs_schema_w72.rs:232:10:
  BASELINE_VERSION not found in source
  ```

## 🧭 Options considered
### Option A (recommended)
- Update the hardcoded path in the tests to read from `crates/tokmd-analysis-types/src/baseline.rs` instead of `lib.rs`.
- Why it fits: It is the most direct and honest fix for the test failure without introducing unnecessary complexity.
- Trade-offs:
  - Structure: Low impact, just updating hardcoded paths.
  - Velocity: Fast.
  - Governance: Restores the ability of the gate checks to catch factual drift.

### Option B
- Update the test utilities to parse Rust syntax (e.g. using `syn`) or resolve module re-exports.
- When to choose it: If constants are frequently moved into deeply nested submodules and maintaining hardcoded paths causes significant friction.
- Trade-offs: Significant scope bloat, slows down xtask compile times, and is overkill for the small number of constants we track.

## ✅ Decision
Option A. It restores the broken governance gate with the simplest, most aligned fix possible.

## 🧱 Changes made (SRP)
- `xtask/tests/docs_schema_w72.rs`: Updated path to `crates/tokmd-analysis-types/src/baseline.rs` for `BASELINE_VERSION`.
- `xtask/tests/docs_w43.rs`: Updated path to `crates/tokmd-analysis-types/src/baseline.rs` for `BASELINE_VERSION`.

## 🧪 Verification receipts
```text
$ cargo test -p xtask --test docs_schema_w72
test baseline_schema_json_version_matches_source ... ok
test schema_md_baseline_version_matches_source ... ok

$ cargo test -p xtask --test docs_w43
test schema_md_baseline_version_matches_source ... ok

$ cargo test -p xtask
test result: ok. 208 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
```

## 🧭 Telemetry
- Change shape: Test fix
- Blast radius: Internal testing/governance only.
- Risk class: Low, test-only.
- Rollback: Revert the PR.
- Gates run: `cargo test -p xtask`

## 🗂️ .jules artifacts
- `.jules/runs/librarian_docs_examples/envelope.json`
- `.jules/runs/librarian_docs_examples/decision.md`
- `.jules/runs/librarian_docs_examples/receipts.jsonl`
- `.jules/runs/librarian_docs_examples/result.json`
- `.jules/runs/librarian_docs_examples/pr_body.md`

## 🔜 Follow-ups
None.