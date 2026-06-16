## 💡 Summary
Fixed a vulnerability in path redaction where logically identical paths containing parent directory segments (`..`) were not normalized correctly before hashing. This could leak information about the directory structure since equivalent paths like `a/b/../c/secret.txt` and `a/c/secret.txt` would produce different hashes.

## 🎯 Why
The path cleaner in `tokmd-format` handles separator normalizations (`\\` to `/`) and ignores `.`/`./` prefixes, but it didn't collapse `..` segments. As a result, paths accessing identical target files produced different BLAKE3 hashes depending on how they traversed directories. By applying standard path component resolution to eliminate `..` traversal, we enforce a much stricter redaction guarantee, preventing accidental leakage.

## 🔎 Evidence
- Path: `crates/tokmd-format/src/redact/mod.rs`
- Finding: `clean_path("a/b/../c/secret.txt")` and `clean_path("a/c/secret.txt")` produced different outputs, yielding `a6fb4284d72856f6.txt` and `e09e20db9035f498.txt` respectively. After the fix, both now map identically without dropping the safe extension suffix.

## 🧭 Options considered
### Option A (recommended)
- Add proper component-based iteration to `clean_path` to resolve `.` and `..` segments before hashing.
- Why it fits: Matches the domain requirement for `tokmd-format::redact`, explicitly avoiding dependence on a physical filesystem. Keeps deterministic processing across systems.
- Trade-offs: Minor string allocations are added to resolve paths but provide strict adherence to deterministic output.

### Option B
- Depend on `std::fs::canonicalize` to fetch the real path before hashing.
- When to choose: Useful if symbolic links must be resolved correctly against physical files.
- Trade-offs: Highly problematic since receipt formatting could occur when the original directory is not available, breaking the pipeline unexpectedly.

## ✅ Decision
Option A. It secures deterministic path hashing uniformly without adding a brittle dependency on file existence during redaction, fully hardening the formatting pipeline.

## 🧱 Changes made (SRP)
- `crates/tokmd-format/src/redact/mod.rs`: Updated `clean_path` to resolve parent path segments (`..`) natively.
- `crates/tokmd-format/tests/test_redaction_leak.rs`: Added deterministic test for parent traversal.

## 🧪 Verification receipts
```text
cargo test -p tokmd-format test_redaction_leak
running 7 tests
test redaction_normalizes_known_compound_archive_suffix_case ... ok
test redaction_drops_suffixes_when_final_extension_is_unsafe ... ok
test redaction_preserves_known_compound_archive_suffix ... ok
test redaction_normalizes_safe_extension_case ... ok
test redaction_preserves_only_final_extension_for_unknown_safe_chains ... ok
test test_redact_path_leak ... ok
test test_redact_path_parent_traversal ... ok
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## 🧭 Telemetry
- Change shape: Core patch
- Blast radius: Formatting pipeline / security boundary (path redactions). Modifies the resulting hashes for paths that formerly included `..`.
- Risk class: Low risk. Fixes determinism guarantees on trust boundaries.
- Rollback: Revert via git on `clean_path`.
- Gates run: targeted cargo build/test on affected crates, contract/snapshot/regression tests, clippy + fmt checks.

## 🗂️ .jules artifacts
- `.jules/runs/run-sentinel-redaction/envelope.json`
- `.jules/runs/run-sentinel-redaction/decision.md`
- `.jules/runs/run-sentinel-redaction/receipts.jsonl`
- `.jules/runs/run-sentinel-redaction/result.json`
- `.jules/runs/run-sentinel-redaction/pr_body.md`

## 🔜 Follow-ups
None.
