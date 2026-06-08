## 💡 Summary
Fixed a path redaction case-normalization issue in `tokmd-format`. Added test to verify identical paths with different extension cases yield the exact same redact/hash output.

## 🎯 Why
Redaction outputs were producing entirely different hash strings when file extensions only differed by casing (e.g., `src/lib.rs` vs `src/lib.RS`), leaking original file casing across the trust boundary.

## 🔎 Evidence
- `crates/tokmd-format/src/redact/mod.rs`
- `test redaction_must_normalize_case_for_safe_extensions_hash ... FAILED`

## 🧭 Options considered
### Option A (recommended)
- Normalize the safe extensions to lowercase before hashing the path.
- Fits this repo and shard: Closes a concrete missed-mutant-style gap strictly within the `tokmd-format` core pipeline, directly preventing information leakage via extension casing.
- Trade-offs: Simple logic addition, adds strict test-backed proof against case leakage.

### Option B
- Modify the `safe_path_extension_suffix` helper to return lower-cased extensions.
- When to choose: If extension normalization responsibility lies in the registry.
- Trade-offs: Requires modification of `clean_path` or the hashed string; doing it purely in `redact_path` before hashing is simpler.

## ✅ Decision
Option A is selected because it directly tackles the hash-input problem exactly where the string is being prepared for hashing within `redact_path`.

## 🧱 Changes made (SRP)
- `crates/tokmd-format/src/redact/mod.rs`
- `crates/tokmd-format/tests/test_redaction_case_leak.rs`

## 🧪 Verification receipts
```text
cargo test -p tokmd-format --test test_redaction_case_leak
test redaction_must_normalize_case_for_safe_extensions_hash ... ok
```

## 🧭 Telemetry
- Change shape: Core bugfix / test coverage
- Blast radius: Output redaction layer only
- Risk class + why: Low risk, deterministic formatting output check fix.
- Rollback: Revert `crates/tokmd-format/src/redact/mod.rs` and the new test file.
- Gates run: `cargo test`, `cargo build`

## 🗂️ .jules artifacts
- `.jules/runs/run-mutant-12345/envelope.json`
- `.jules/runs/run-mutant-12345/decision.md`
- `.jules/runs/run-mutant-12345/receipts.jsonl`
- `.jules/runs/run-mutant-12345/result.json`
- `.jules/runs/run-mutant-12345/pr_body.md`

## 🔜 Follow-ups
None.
