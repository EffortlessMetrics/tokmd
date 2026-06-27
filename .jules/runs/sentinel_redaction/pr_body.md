## 💡 Summary
This PR fixes a path normalization vulnerability in the core pipeline's redaction logic. The `clean_path` function previously failed to resolve `..` (parent directory) segments, causing identical logical paths to produce different redaction hashes and potentially leaking directory traversal structures in outputs.

## 🎯 Why
The core redaction pipeline must strictly guarantee that logically identical paths produce deterministic hashes to prevent directory structure leakage across trust boundaries. The prior implementation used a simple string replacement approach that ignored `..` traversal segments, violating the `security-boundary` gate profile expectations for deterministic safety.

## 🔎 Evidence
- File: `crates/tokmd-format/src/redact/mod.rs`
- Finding: `clean_path("src/secret/passwords.txt")` produced a different hash than `clean_path("src/public/../../secret/passwords.txt")`.
- Receipt:
```text
test redaction_resolves_parent_directory_leak ... FAILED
thread 'redaction_resolves_parent_directory_leak' panicked at crates/tokmd-format/tests/test_redaction_leak.rs:58:5:
assertion `left == right` failed: Directory traversal leak: paths were not resolved properly
```

## 🧭 Options considered
### Option A (recommended)
- Improve the `clean_path` function using a stack-based resolution to properly normalize `..` segments along with handling absolute paths.
- Fits the `core-pipeline` shard and directly addresses the Sentinel persona's #1 target ranking for redaction correctness and leakage prevention.
- Trade-offs:
  - Structure: Closes a directory traversal format leakage gap.
  - Velocity: Quick, strictly localized within the `tokmd-format` redaction module.
  - Governance: High alignment with `security-boundary` invariants.

### Option B
- Document the traversal format leak as a known limitation in a learning PR.
- Choose this only if addressing the issue risks breaking essential cross-boundary literal formats.
- Trade-offs: Violates the explicit instruction to land one security-significant hardening improvement when a clean localized fix exists.

## ✅ Decision
Proceed with Option A. The `clean_path` function is the canonical source for hash string generation to avoid leakage. Using a stack-based path resolution directly addresses the vulnerability while preserving cross-platform formatting determinism.

## 🧱 Changes made (SRP)
- `crates/tokmd-format/src/redact/mod.rs`: Rewrote `clean_path` to use a stack-based approach that correctly resolves `..` segments and handles absolute/relative invariants. Added parent directory test cases.
- `crates/tokmd-format/tests/test_redaction_leak.rs`: Added a vulnerability test `redaction_resolves_parent_directory_leak` to verify proper structural path resolution.

## 🧪 Verification receipts
```text
$ cargo test --test test_redaction_leak
running 7 tests
test redaction_normalizes_safe_extension_case ... ok
test redaction_drops_suffixes_when_final_extension_is_unsafe ... ok
test redaction_preserves_known_compound_archive_suffix ... ok
test redaction_normalizes_known_compound_archive_suffix_case ... ok
test redaction_preserves_only_final_extension_for_unknown_safe_chains ... ok
test redaction_resolves_parent_directory_leak ... ok
test test_redact_path_leak ... ok
test result: ok. 7 passed; 0 failed

$ cargo test -p tokmd-format --lib
test result: ok. 145 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## 🧭 Telemetry
- Change shape: Hardening
- Blast radius: Internal redaction outputs within `tokmd-format`
- Risk class: Low (purely string processing improvements without IO/API footprint changes)
- Rollback: Revert the PR to restore the prior string replacement approach.
- Gates run: `cargo test -p tokmd-format`, `security-boundary` fallback expectations

## 🗂️ .jules artifacts
- `.jules/runs/sentinel_redaction/envelope.json`
- `.jules/runs/sentinel_redaction/decision.md`
- `.jules/runs/sentinel_redaction/receipts.jsonl`
- `.jules/runs/sentinel_redaction/result.json`
- `.jules/runs/sentinel_redaction/pr_body.md`

## 🔜 Follow-ups
None.
