## 🧭 Options considered
### Option A (recommended)
- Add a new integration test file `crates/tokmd-format/tests/test_strip_prefix_redaction.rs` to assert that `strip_prefix` is correctly redacted in JSON/JSONL output when `redact` is `Paths` or `All`, and preserved when `redact` is `None`.
- This targets a key security/redaction path, ensuring the `strip_prefix` field in the meta envelope cannot leak sensitive internal directory structures.
- Trade-offs: Structure is solid as it lives in integration tests; Velocity is fast since the test already passes against the current logic, closing an assertion gap without refactoring; Governance is unaffected.

### Option B
- Modify `format_tests.rs` or `snapshot_golden_w54.rs` to include strip_prefix redaction tests.
- When to choose: If we prefer fewer test files and want to pile more assertions into the massive existing snapshot or format tests.
- Trade-offs: Makes the existing monolithic test files harder to read. A standalone test file clearly signals the intent and boundary of the security/redaction check.

## ✅ Decision
Option A. Adding `test_strip_prefix_redaction.rs` provides direct, behavioral proof that the `strip_prefix_redacted` flag and `strip_prefix` payload are handled correctly under different redaction modes. It meets the Mutant persona goal of strengthening behavioral assertions around meaningful code paths without polluting existing test monolithic suites.
