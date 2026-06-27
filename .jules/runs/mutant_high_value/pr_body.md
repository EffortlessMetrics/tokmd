## 💡 Summary
Replaced the string-manipulation-based path cleaning logic in `tokmd-format` with a robust stack-based normalization that properly handles parent (`..`) segments. Resolved a clippy warning regarding empty string comparisons and added a specific unit test to prove parent traversal fixes.

## 🎯 Why
The old `clean_path` logic in `crates/tokmd-format/src/redact/mod.rs` only handled stripping leading `./` and interior `/./` segments, leaving `..` unresolved. This could lead to logically identical paths producing different hashes, potentially leaking the presence of directory traversal or differing directory structures in redacted outputs. By properly collapsing parent segments using a stack, we guarantee that identical resolved paths produce deterministic hashes.

## 🔎 Evidence
- `crates/tokmd-format/src/redact/mod.rs` path normalization logic was weak.
- Test `clean_path("crates/tokmd/src/../../tokmd-format/src/lib.rs")` correctly resolves to `crates/tokmd-format/src/lib.rs` under the new logic instead of retaining `..`.
- Clippy flagged `part == ""` comparisons, which were changed to `part.is_empty()`.

## 🧭 Options considered
### Option A (recommended)
- Reimplement `clean_path` using `s.replace('\\', "/")` and a stack (`Vec<&str>`) to properly resolve `.` and `..` segments, preserving leading absolute slashes.
- Fits this repo and shard well, solidifying path redaction logic in `core-pipeline`.
- Trade-offs: Structure: High (correctness), Velocity: Neutral, Governance: Met.

### Option B
- Only fix the clippy warning and attempt a regex or substring replacement for `..`.
- When to choose it instead: If the paths were guaranteed never to contain parent traversal.
- Trade-offs: Error-prone and doesn't handle deep traversal like `foo/../../bar` correctly.

## ✅ Decision
Option A was chosen because a robust stack-based algorithm correctly guarantees path normalization across all operating systems without edge case failures. It guarantees deterministic redaction hashing and resolves the `clippy::comparison_to_empty` warning. Added the `test_clean_path_parent_resolution` test to fulfill mutation proving guidelines.

## 🧱 Changes made (SRP)
- Modified `crates/tokmd-format/src/redact/mod.rs` to replace the `clean_path` function and resolve a `clippy` warning.
- Added `test_clean_path_parent_resolution` test to `crates/tokmd-format/src/redact/mod.rs`.

## 🧪 Verification receipts
```text
$ cargo clippy -p tokmd-format -- -D warnings
    Checking tokmd-format v1.13.1 (/app/crates/tokmd-format)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.87s

$ CI=true cargo test -p tokmd-format
test redact::tests::test_clean_path_parent_resolution ... ok
test redaction_normalizes_safe_extension_case ... ok
test test_redact_path_leak ... ok
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## 🧭 Telemetry
- Change shape: Fix
- Blast radius: API (internal formatting `clean_path` affects redaction hashes)
- Risk class: Low, only impacts hash stability for paths with `..` in receipts or reports.
- Rollback: Revert to previous string replace logic.
- Gates run: `cargo clippy -p tokmd-format -- -D warnings`, `CI=true cargo test -p tokmd-format`.

## 🗂️ .jules artifacts
- `.jules/runs/mutant_high_value/envelope.json`
- `.jules/runs/mutant_high_value/decision.md`
- `.jules/runs/mutant_high_value/receipts.jsonl`
- `.jules/runs/mutant_high_value/result.json`
- `.jules/runs/mutant_high_value/pr_body.md`

## 🔜 Follow-ups
None.
