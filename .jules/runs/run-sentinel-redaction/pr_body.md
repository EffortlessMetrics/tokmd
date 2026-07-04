## 💡 Summary
Fixes an issue where `strip_prefix` would leak path extensions even when redaction was enabled. Replaces `redact_path` with `short_hash` for prefix processing to guarantee fully opaque directory redaction.

## 🎯 Why
Redaction mode in the export pipeline (via `redact_path`) correctly preserves safe extensions for full file paths to aid in LLM processing. However, when applied to `strip_prefix` (which usually represents a directory root, like `src/secret.rs` mimicking a file), it inadvertently leaks characters by assuming the last segment contains an extension. A prefix boundary should never leak its content under redaction.

## 🔎 Evidence
- `crates/tokmd-core/src/receipts.rs` mapped `strip_prefix` using `tokmd_format::redact_path(p)`.
- `crates/tokmd-format/src/export/json.rs` mapped `strip_prefix` using `redact_path(...)`.
- `crates/tokmd-format/src/export/jsonl.rs` mapped `strip_prefix` using `redact_path(...)`.

## 🧭 Options considered
### Option A (recommended)
- what it is: Use `short_hash` directly instead of `redact_path` when redacting the `strip_prefix` field in receipts and export metadata.
- why it fits this repo and shard: It securely hardens the boundary of receipt data in the core pipeline, preventing a redaction correctness leak.
- trade-offs: Safe, straightforward hardening with no behavioral changes to actual code path formatting.

### Option B
- what it is: Add a new mode to `redact_path` to ignore extensions.
- when to choose it instead: If many surfaces required blind redaction with the `redact_path` signature.
- trade-offs: More complex, creates unnecessary API surface when `short_hash` already does exactly this.

## ✅ Decision
Option A was chosen because `short_hash` correctly and robustly provides an opaque representation for prefixes, completely mitigating the extension-leak vector.

## 🧱 Changes made (SRP)
- `crates/tokmd-core/src/receipts.rs`: Replace `redact_path` with `short_hash` for `strip_prefix` in `build_export_receipt`.
- `crates/tokmd-format/src/export/json.rs`: Update `strip_prefix` processing in `write_export_json` to use `short_hash` instead of `redact_path`.
- `crates/tokmd-format/src/export/jsonl.rs`: Update `strip_prefix` processing in `write_export_jsonl_to_file` to use `short_hash` instead of `redact_path`.

## 🧪 Verification receipts
```text
running 1 test
test test_write_export_cyclonedx_honors_redact_mode ... ok

running 6 tests
test redaction_drops_suffixes_when_final_extension_is_unsafe ... ok
test redaction_preserves_known_compound_archive_suffix ... ok
test redaction_normalizes_known_compound_archive_suffix_case ... ok
test redaction_normalizes_safe_extension_case ... ok
test redaction_preserves_only_final_extension_for_unknown_safe_chains ... ok
test test_redact_path_leak ... ok

running 10 tests
test crates/tokmd-core/src/lib.rs - (line 42) ... ok
test crates/tokmd-core/src/lib.rs - (line 24) ... ok
test crates/tokmd-core/src/workflows/export.rs - workflows::export::export_workflow_from_inputs (line 62) ... ok
test crates/tokmd-core/src/ffi/mod.rs - ffi::run_json (line 60) ... ok
```

## 🧭 Telemetry
- Change shape: Hardening
- Blast radius: Output receipts (`strip_prefix` field)
- Risk class: Low, only changes output of redacted prefix strings
- Rollback: Revert PR
- Gates run: `cargo test -p tokmd-core`, `cargo test -p tokmd-format`, `cargo clippy`, `cargo fmt`

## 🗂️ .jules artifacts
- `.jules/runs/run-sentinel-redaction/envelope.json`
- `.jules/runs/run-sentinel-redaction/decision.md`
- `.jules/runs/run-sentinel-redaction/receipts.jsonl`
- `.jules/runs/run-sentinel-redaction/result.json`
- `.jules/runs/run-sentinel-redaction/pr_body.md`

## 🔜 Follow-ups
None
