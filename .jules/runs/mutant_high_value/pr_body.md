## 💡 Summary
Extracted the inline sorting logic from `tokmd-model`'s core into public deterministic sort functions and updated `determinism_w66.rs` to use them directly. This closes a large testing gap where `determinism_w66.rs` was testing its own duplicated sorting functions rather than the actual sorting logic used in production.

## 🎯 Why
The `determinism_w66.rs` test suite previously redefined `sort_lang_rows`, `sort_module_rows`, and `sort_file_rows`. This means the test suite was asserting determinism against its *own* logic, rather than the library's actual behavior. Any mutation in the production library's inline sorting closures would go unnoticed. By centralizing these, we close a massive mutation coverage gap around deterministic sorting.

## 🔎 Evidence
- File: `crates/tokmd-model/tests/determinism_w66.rs`
- Finding: The test defined duplicate sort functions (e.g., `fn sort_lang_rows(rows: &mut [LangRow]) { ... }`) that exactly mirrored the inline closures in `build_model`.
- Receipt: Tests passing successfully using the actual public API (`tokmd_model::sort_lang_rows`) after extraction.

## 🧭 Options considered
### Option A (recommended)
- Extract duplicate test sorting functions into the `tokmd-model` public API and call them both from the library and the tests.
- Fits the `core-pipeline` shard and fulfills the Mutant goal of improving mutation-style test assertions.
- Trade-offs: Increases the public API surface slightly, but massively improves test confidence.

### Option B
- Add heavy end-to-end integration tests that call `build_model` and `create_export_data_from_rows` with permuted inputs to test determinism through the public interface without extracting sort methods.
- Slower tests, and doesn't remove the misleading duplicated test functions that give false security.
- Trade-offs: Lower velocity, higher complexity.

## ✅ Decision
Option A. It's cleaner, removes duplicated test logic, and effectively closes the mutation gap in `determinism_w66.rs`.

## 🧱 Changes made (SRP)
- `crates/tokmd-model/src/lib.rs`
- `crates/tokmd-model/tests/determinism_w66.rs`

## 🧪 Verification receipts
```text
cargo build
cargo test -p tokmd-model
cargo clippy -p tokmd-model -- -D warnings
cargo fmt -- --check
```

## 🧭 Telemetry
- Change shape: Extract function, test cleanup
- Blast radius: API (added 3 new functions to `tokmd-model`), Tests
- Risk class: Low, pure extraction of existing inline logic.
- Rollback: Revert the PR.
- Gates run: build, test, clippy, fmt

## 🗂️ .jules artifacts
- `.jules/runs/mutant_high_value/envelope.json`
- `.jules/runs/mutant_high_value/decision.md`
- `.jules/runs/mutant_high_value/receipts.jsonl`
- `.jules/runs/mutant_high_value/result.json`
- `.jules/runs/mutant_high_value/pr_body.md`

## 🔜 Follow-ups
None
