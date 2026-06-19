## 💡 Summary
Updated `clean_path` in the `tokmd-format` crate to correctly resolve parent directory (`..`) segments and normalize path separators. Added comprehensive integration tests to ensure deterministic path redaction and prevent directory structure leakage.

## 🎯 Why
The previous implementation of `clean_path` stripped redundant paths (`./`, `/./`) but failed to resolve `..` segments. This meant that logically identical paths like `crates/tokmd/../foo/lib.rs` and `crates/foo/lib.rs` produced different hashes during redaction. Correctly resolving these segments guarantees deterministic hashes and prevents directory structure leakage in redacted output, which is a critical security-boundary hardening measure.

## 🔎 Evidence
Minimal proof:
- `crates/tokmd-format/src/redact/mod.rs`
- Observed behavior: `redact_path("crates/tokmd/../foo/lib.rs")` and `redact_path("crates/foo/lib.rs")` returned different hashes.
- Test receipt: The new `test_clean_path.rs` demonstrates that they now map to the exact same hash correctly.

## 🧭 Options considered
### Option A (recommended)
- Update `clean_path` to split paths by `/` and use a stack (`Vec`) to manually resolve `..` segments.
- Fits this repo and shard as `tokmd-format` handles redaction logic, and solving it centrally ensures that all dependent paths are correctly redacted.
- Trade-offs: Minor string allocations are needed to rebuild the path, but the frequency of these operations is low enough that the performance impact is negligible compared to the determinism and security benefits.

### Option B
- Use the standard library `std::path::Path::canonicalize` or similar.
- When to choose it instead: If disk I/O was acceptable for the redaction step.
- Trade-offs: Disk access is too slow and canonicalize requires the file to exist on the filesystem, which is not guaranteed for all redacted paths (e.g., virtual files or deleted files).

## ✅ Decision
Option A. It explicitly resolves the determinism issue without requiring file I/O, maintaining the fast, stateless nature of the redaction system. It fulfills the core objective of the `mutation` gate to close concrete missed-mutant gaps with deterministic behavioral tests.

## 🧱 Changes made (SRP)
- Modified `crates/tokmd-format/src/redact/mod.rs` to implement robust path normalization using a stack.
- Added `crates/tokmd-format/tests/test_clean_path.rs` with new integration tests.

## 🧪 Verification receipts
```text
running 7 tests
test test_redact_path_complex_absolute ... ok
test test_redact_path_complex_normalization ... ok
test test_redact_path_dot_double_slash ... ok
test test_redact_path_double_slash ... ok
test test_redact_path_normalizes_parent_segments ... ok
test test_redact_path_normalizes_parent_segments_at_root ... ok
test test_redact_path_preserves_absolute_paths ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## 🧭 Telemetry
- Change shape: Core formatting logic patch with corresponding tests.
- Blast radius: API (redacted path outputs).
- Risk class: Low, isolated to path normalization and redaction formatting.
- Rollback: Revert the PR if unexpected pathing structures are incorrectly flattened.
- Gates run: `cargo build`, `cargo test`, `cargo fmt`, `cargo clippy`.

## 🗂️ .jules artifacts
- `.jules/runs/run-mutant-1/envelope.json`
- `.jules/runs/run-mutant-1/decision.md`
- `.jules/runs/run-mutant-1/receipts.jsonl`
- `.jules/runs/run-mutant-1/result.json`
- `.jules/runs/run-mutant-1/pr_body.md`

## 🔜 Follow-ups
None.
