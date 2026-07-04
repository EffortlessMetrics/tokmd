# Decision

## Option A (Recommended)
Remove the bug that leaks the strip_prefix flag even when it is redacted in `tokmd-core` and `tokmd-format`. In `tokmd-core::receipts::build_export_receipt`, when `should_redact` is true, the `strip_prefix` should be transformed through `tokmd_format::redact_path`. But the result of `tokmd_format::redact_path` leaks the filename extension. Since `strip_prefix` is a prefix, we should use `tokmd_format::short_hash` instead.
There's also a similar place in `tokmd-format::export::json::write_export_json` and `tokmd-format::export::jsonl::write_export_jsonl_to_file`.

## Option B
Do not fix the leak but mark it as a friction item or persona note. This goes against the Sentinel mission.

## Decision
I will go with Option A because it fulfills the Sentinel mission to fix a redaction correctness and leakage prevention issue. I will modify `tokmd-core::receipts::build_export_receipt`, `crates/tokmd-format/src/export/json.rs` and `crates/tokmd-format/src/export/jsonl.rs` to use `short_hash` instead of `redact_path` for `strip_prefix` when redaction is enabled.
