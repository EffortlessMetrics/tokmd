## 💡 Summary
Added an integration test to assert that `strip_prefix` in the export meta envelope is properly redacted (hashed) when `RedactMode` is `Paths` or `All`, and correctly signals the `strip_prefix_redacted` flag. This closes a behavioral testing gap without modifying production code.

## 🎯 Why
The `strip_prefix` field in the meta envelope contains sensitive internal directory structures. While the production redaction logic was correctly covering this field across JSON and JSONL exports, there was no standalone behavioral integration test asserting that the meta envelope properties (`strip_prefix` and `strip_prefix_redacted`) were working in concert. Adding targeted tests prevents regressions on this security/redaction boundary.

## 🔎 Evidence
- `crates/tokmd-format/tests/test_strip_prefix_redaction.rs`
- Confirmed correct behavior and passing state via `cargo test --test test_strip_prefix_redaction` covering JSON/JSONL across `RedactMode::Paths`, `All`, and `None`.

## 🧭 Options considered
### Option A (recommended)
- Add a standalone integration test file `crates/tokmd-format/tests/test_strip_prefix_redaction.rs`.
- Why it fits: Standalone behavioral integration tests are ideal for the `core-pipeline` shard's `mutation` gate, as they cleanly assert a specific sub-system contract without tangling up monolithic snapshot tests.
- Trade-offs: Structure/Governance are unaffected. Velocity is high.

### Option B
- Shove the new test checks into existing massive monolithic test files like `format_tests.rs`.
- When to choose: If file counts are heavily constrained.
- Trade-offs: Degrades readability and hides the explicit purpose of the redaction test.

## ✅ Decision
Option A. Adding a standalone test explicitly proves the `strip_prefix` and `strip_prefix_redacted` behaviors under different modes and perfectly fulfills the Mutant persona's mandate to strengthen testing around high-value surfaces.

## 🧱 Changes made (SRP)
- `crates/tokmd-format/tests/test_strip_prefix_redaction.rs`

## 🧪 Verification receipts
```text
cargo test --test test_strip_prefix_redaction
running 3 tests
test strip_prefix_is_preserved_when_mode_is_none ... ok
test strip_prefix_is_redacted_for_jsonl_when_mode_is_paths_or_all ... ok
test strip_prefix_is_redacted_when_mode_is_paths_or_all ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

cargo fmt -- --check
cargo clippy -- -D warnings
CI=true cargo test -p tokmd-format --verbose
```

## 🧭 Telemetry
- Change shape: New integration tests only.
- Blast radius: Testing API. No production dependencies or implementations altered.
- Risk class: Zero risk. Test-only addition.
- Rollback: Revert the new test file addition.
- Gates run: targeted `cargo test`, `cargo fmt`, `cargo clippy`, and `cargo test -p tokmd-format --verbose`.

## 🗂️ .jules artifacts
- `.jules/runs/mutant_high_value/envelope.json`
- `.jules/runs/mutant_high_value/decision.md`
- `.jules/runs/mutant_high_value/receipts.jsonl`
- `.jules/runs/mutant_high_value/result.json`
- `.jules/runs/mutant_high_value/pr_body.md`

## 🔜 Follow-ups
None.
