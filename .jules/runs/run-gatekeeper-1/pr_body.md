## 💡 Summary
Hardened the output normalization regexes in determinism tests to tolerate optional whitespace after colons in formatted JSON (e.g., `"generated_at_ms": 123`). This eliminates a brittle failure mode where tests flag determinism drifts simply due to serializer spacing changes.

## 🎯 Why
The determinism regression suite relies on regex to scrub volatile fields like `generated_at_ms` and `tool.version` before comparing outputs byte-for-byte. The old regexes assumed no whitespace (`r#""generated_at_ms":\d+"#`), causing them to fail to match if a serializer added a space, which would then fail the byte-for-byte equality check.

## 🔎 Evidence
- Found in: `crates/tokmd/tests/determinism.rs` and other integration test files.
- Observed behavior: Normalization string replace failed if whitespace existed.
- Receipt: `cargo test --test determinism_regression` passing after fixing the regexes to use `\s*`.

## 🧭 Options considered
### Option A (recommended)
- **What it is:** Update the regex to `\s*`.
- **Why it fits this repo and shard:** Fixes the tests directly and immediately while retaining byte-for-byte guarantees over the raw string output format (including alphabetical ordering of keys).
- **Trade-offs:** Minimal velocity cost, protects structural test integrity.

### Option B
- **What it is:** Parse output with `serde_json`, nullify fields, and reserialize.
- **When to choose it instead:** If the structure of JSON became incredibly dynamic and non-deterministic, making regex matching impossible.
- **Trade-offs:** Destroys implicit format guarantees (like BTreeMap sorted keys serialization) that tests use raw string comparison to protect.

## ✅ Decision
Option A. Tolerating whitespace via regex retains the strict raw string output guarantees of the determinism tests.

## 🧱 Changes made (SRP)
- `crates/tokmd/tests/cli_snapshot_golden.rs`
- `crates/tokmd/tests/determinism.rs`
- `crates/tokmd/tests/integration_w70.rs`

## 🧪 Verification receipts
```text
running 26 tests
test analyze_json_no_backslash_in_path_fields ... ok
test analyze_receipt_has_stable_top_level_keys ... ok
test analyze_derived_keys_stable_and_sorted ... ok
test analyze_receipt_json_byte_identical_across_runs ... ok
test export_csv_byte_identical ... ok
test export_json_no_backslash_in_path_or_module ... ok
test analyze_receipt_markdown_deterministic ... ok
...
test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.52s
```

## 🧭 Telemetry
- Change shape: Test infrastructure regex loosening
- Blast radius: `crates/tokmd/tests/`
- Risk class: Low - Test-only update
- Rollback: `git restore crates/tokmd/tests/`
- Gates run: `cargo test --test determinism_regression`

## 🗂️ .jules artifacts
- `.jules/runs/run-gatekeeper-1/envelope.json`
- `.jules/runs/run-gatekeeper-1/decision.md`
- `.jules/runs/run-gatekeeper-1/result.json`
- `.jules/runs/run-gatekeeper-1/pr_body.md`

## 🔜 Follow-ups
None.
