## 💡 Summary
This is a learning PR. We investigated a potential trust-boundary data leak where mixed-case file extensions could bypass case-normalization during redaction in `tokmd-format`. Our tests confirmed the existing logic already safely normalizes these to lowercase, meaning the boundary is secure.

## 🎯 Why
The prompt indicated that `tokmd-format` path redaction must normalize known safe extensions to lowercase rather than preserving mixed-case input. However, after exploring `crates/tokmd-format/src/redact/extensions.rs`, we determined that the statically-defined `SAFE_PATH_EXTENSIONS` already enforce a strict lowercase output policy. To prevent landing a fake fix, we are emitting a learning PR.

## 🔎 Evidence
- `crates/tokmd-format/src/redact/extensions.rs`
- Tests run locally using `cargo test -p tokmd-format --test test_redaction_leak`
- Both single extensions (`file.jSoN` -> `.json`) and compound suffixes (`archive.tAr.Gz` -> `.tar.gz`) were correctly normalized without any code modifications.

## 🧭 Options considered
### Option A (recommended)
- Emit a learning PR and friction item.
- Fits the shard and persona because Sentinel mandates high-confidence improvements, and explicitly forbids forcing fake fixes when the boundary is already hardened.
- Trade-offs: Structure is preserved without unnecessary commits; Velocity is maintained.

### Option B
- Introduce redundant normalization functions.
- Choose this only if the current functions failed to output strictly lowercase extensions.
- Trade-offs: Would add needless complexity and hallucinate a fix for an issue that doesn't exist.

## ✅ Decision
Option A. The `tokmd-format` path redaction already properly normalizes mixed-case input.

## 🧱 Changes made (SRP)
- Added a friction item regarding the verified-safe redaction leak.
- Added a Sentinel persona note.

## 🧪 Verification receipts
```text
running 8 tests
test redaction_does_not_leak_mixed_case ... ok
test redaction_drops_suffixes_when_final_extension_is_unsafe ... ok
test redaction_normalizes_safe_extension_case ... ok
test redaction_normalizes_known_compound_archive_suffix_case ... ok
test redaction_preserves_known_compound_archive_suffix ... ok
test redaction_preserves_only_final_extension_for_unknown_safe_chains ... ok
test test_compound_suffix_mixed_case ... ok
test test_redact_path_leak ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## 🧭 Telemetry
- Change shape: Learning PR
- Blast radius: None (No production code changes)
- Risk class: Low
- Rollback: N/A
- Gates run: targeted cargo test

## 🗂️ .jules artifacts
- `.jules/runs/sentinel_redaction/envelope.json`
- `.jules/runs/sentinel_redaction/decision.md`
- `.jules/runs/sentinel_redaction/receipts.jsonl`
- `.jules/runs/sentinel_redaction/result.json`
- `.jules/runs/sentinel_redaction/pr_body.md`
- `.jules/friction/open/sentinel_redaction.md`
- `.jules/personas/sentinel/notes/redaction_case_normalization.md`

## 🔜 Follow-ups
None.
