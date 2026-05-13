## 💡 Summary
Ensured that path redaction produces case-insensitive output extensions regardless of the input case. Redaction correctness and leakage prevention have been improved to prevent exposing information about original file casing.

## 🎯 Why
In `tokmd-format/src/redact/mod.rs`, the `redact_path` function hashes the path but attempts to preserve a common "safe" extension (like `.rs` or `.json`). It checks whether the extension is in an allowlist case-insensitively, but it appended the *original* extension to the redacted path rather than a normalized lower-cased version. This meant that `file.JSON` would result in a redacted path ending with `.JSON`, which is an unintended trust-boundary data leak of case information.

## 🔎 Evidence
- **File path**: `crates/tokmd-format/src/redact/extensions.rs` and `crates/tokmd-format/src/redact/mod.rs`
- **Observed behavior**: Calling `redact_path("file.JSON")` incorrectly produced an extension of `.JSON`.
- **Receipt**: `cargo test -p tokmd-format test_redaction_leak` failed on the new test `case_sensitive_extension_leak` before the fix.

## 🧭 Options considered
### Option A (recommended)
- **What it is**: Ensure `safe_path_extension` returns the static string from the lowercase allowlist rather than the original input string.
- **Why it fits this repo and shard**: Fixing this falls under the `security-boundary` gate profile by ensuring proper boundary truncation (redaction correctness). Returning `&'static str` avoids string allocations.
- **Trade-offs**:
  - **Structure**: It creates cleaner and more deterministic outputs without allocations.
  - **Velocity**: Trivial code change.
  - **Governance**: Fixes an unaddressed minor trust-boundary leak.

### Option B
- **What it is**: Do nothing and create a learning PR.
- **When to choose it instead**: If the behavior was an accepted limitation of the extension or deemed irrelevant to security boundary.
- **Trade-offs**: This leaves a minor leak of original path casing.

## ✅ Decision
Option A was selected to fully close the extension-casing trust boundary leak in `tokmd-format` efficiently using static slices.

## 🧱 Changes made (SRP)
- `crates/tokmd-format/src/redact/extensions.rs`: Changed `safe_path_extension` and `safe_path_extension_suffix` to return `Option<&'static str>` and `Option<Vec<&'static str>>` from the allowlists directly, rather than returning the input string slice.
- `crates/tokmd-format/tests/test_redaction_leak.rs`: Added the `case_sensitive_extension_leak` test.

## 🧪 Verification receipts
```text
{"command": "cargo test -p tokmd-format test_redaction_leak", "output": "test result: FAILED. 0 passed; 1 failed; 0 ignored"}
{"command": "cargo test -p tokmd-format test_redaction_leak", "output": "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured"}
{"command": "cargo test -p tokmd-format", "output": "test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured"}
{"command": "cargo build -p tokmd-format --verbose", "output": "Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.91s"}
```

## 🧭 Telemetry
- **Change shape**: Narrow boundary fix
- **Blast radius**: Minimal. Only impacts redacted path output for mixed-case extensions.
- **Risk class + why**: Low risk. Redaction is already a one-way lossy hash, fixing case determinism improves its properties without compromising downstream processing.
- **Rollback**: Easily revertible.
- **Gates run**: `cargo test -p tokmd-format`, `cargo clippy -p tokmd-format -- -D warnings`, `cargo fmt -- --check`

## 🗂️ .jules artifacts
- `.jules/runs/run-sentinel-redact-1/envelope.json`
- `.jules/runs/run-sentinel-redact-1/decision.md`
- `.jules/runs/run-sentinel-redact-1/receipts.jsonl`
- `.jules/runs/run-sentinel-redact-1/result.json`
- `.jules/runs/run-sentinel-redact-1/pr_body.md`

## 🔜 Follow-ups
None.
