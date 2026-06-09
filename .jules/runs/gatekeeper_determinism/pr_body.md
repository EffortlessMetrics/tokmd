## 💡 Summary
Tightened path redaction determinism by normalizing preserved file extensions to lowercase. Added a regression test to ensure original file casing is not leaked across the redaction boundary.

## 🎯 Why
When redacting paths using `redact_path`, the original case of a safe file extension was preserved (e.g. `file.JSON` became `<hash>.JSON`). This was a deterministic sharp edge and a minor trust boundary leak, exposing original path casing metadata. Normalizing the hash prefix and extensions to lowercase locks in fully deterministic, case-insensitive output.

## 🔎 Evidence
- `crates/tokmd-format/tests/test_redaction_leak.rs`
- `crates/tokmd-format/src/redact/mod.rs`
- Observed `file.JSON` hashing to a different output casing than `file.json`.

## 🧭 Options considered
### Option A (recommended)
- Normalize path components and the final preserved extension to lowercase during redaction.
- Why: Fixes determinism sharp edge in path redaction. Fits `contracts-determinism` shard perfectly.
- Trade-offs:
  - Structure: Prevents case variation from altering deterministic hash shape.
  - Velocity: Extremely low risk patch.
  - Governance: Tightens the format contract to ensure zero structural metadata leakage.

### Option B
- Document the behavior without modifying it.
- When to choose: If backwards compatibility of exact hash string representations was critical for the `JSON` vs `json` casing.
- Trade-offs: Leaves a known leakage vector in place.

## ✅ Decision
Option A was chosen to protect the redaction contract and ensure strict deterministic output.

## 🧱 Changes made (SRP)
- `crates/tokmd-format/src/redact/mod.rs` - Normalized `cleaned` path to lowercase for hashing and ensured the preserved extensions are lowercase.
- `crates/tokmd-format/tests/test_redaction_leak.rs` - Added `test_redact_path_leaks_original_case` proof surface to guarantee this leak is bounded.

## 🧪 Verification receipts
```text
cargo test -p tokmd-format --test test_redaction_leak
test redaction_drops_suffixes_when_final_extension_is_unsafe ... ok
test redaction_normalizes_safe_extension_case ... ok
test redaction_preserves_known_compound_archive_suffix ... ok
test redaction_normalizes_known_compound_archive_suffix_case ... ok
test redaction_preserves_only_final_extension_for_unknown_safe_chains ... ok
test test_redact_path_leak ... ok
test test_redact_path_leaks_original_case ... ok

cargo test -p tokmd-format redact
test result: ok. 31 passed; 0 failed

cargo fmt -- --check
cargo clippy -- -D warnings
```

## 🧭 Telemetry
- Change shape: Hardening
- Blast radius: Internal redaction outputs in `tokmd-format`.
- Risk class: Low, only normalizes string outputs that were already meant to be redacted.
- Rollback: Revert the `.to_ascii_lowercase()` normalization.
- Gates run: `cargo test`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`.

## 🗂️ .jules artifacts
- `.jules/runs/gatekeeper_determinism/envelope.json`
- `.jules/runs/gatekeeper_determinism/decision.md`
- `.jules/runs/gatekeeper_determinism/receipts.jsonl`
- `.jules/runs/gatekeeper_determinism/result.json`
- `.jules/runs/gatekeeper_determinism/pr_body.md`

## 🔜 Follow-ups
None.
