## 💡 Summary
Hardened `tokmd-format` JSON and JSONL exports to correctly redact structural array `module_roots` when `RedactMode::All` is set. Previously, `module_roots` was merely cloned and bypassed redaction.

## 🎯 Why
When using `RedactMode::All`, users expect both filenames and module paths to be completely anonymized before structural metadata is exported. However, the `ExportArgsMeta` payload exported with JSON and JSONL datasets was passing `module_roots` in plaintext, leaking directory names.

## 🔎 Evidence
- File: `crates/tokmd-format/src/lib.rs`
- Finding: `module_roots` in `write_export_jsonl` and `write_export_json` were being populated with `export.module_roots.clone()` instead of applying `short_hash()` when `RedactMode::All` was active.

## 🧭 Options considered
### Option A (recommended)
- what it is: Conditionally iterate over `export.module_roots` and apply `short_hash` if `RedactMode::All`.
- why it fits this repo and shard: Fixes a security regression in the `core-pipeline` shard (`tokmd-format`) directly relating to redaction expectations.
- trade-offs: Minor computational overhead on export to hash structural arrays.

### Option B
- what it is: Remove `module_roots` from exports completely if `RedactMode::All`.
- when to choose it instead: If the cardinality of `module_roots` provides a structural risk.
- trade-offs: Breaks downstream analysis tools expecting the array to be present and sized correctly.

## ✅ Decision
Option A was chosen. Preserving the structure of `module_roots` while redacting the contents using `short_hash` aligns with how the actual `.module` columns are handled in the CSV/TSV table outputs.

## 🧱 Changes made (SRP)
- `crates/tokmd-format/src/lib.rs`
  - Patched `write_export_jsonl` to redact `module_roots` under `RedactMode::All`.
  - Patched `write_export_json` to redact `module_roots` inside `ExportArgsMeta` and `ExportData` under `RedactMode::All`.

## 🧪 Verification receipts
```text
{"command": "python3 patch.py", "outcome": "Success, applied redaction to module_roots"}
{"command": "cargo build -p tokmd-format", "outcome": "Success, compiles without errors"}
{"command": "cd crates/tokmd-format && cargo test", "outcome": "Success, all tests passed"}
{"command": "cargo fmt -- --check", "outcome": "Success"}
{"command": "cargo clippy -p tokmd-format -- -D warnings", "outcome": "Success"}
```

## 🧭 Telemetry
- Change shape: Hardening patch
- Blast radius: `tokmd-format` (JSON/JSONL outputs only)
- Risk class: Low risk. Does not break downstream schemas, only hashes strings in arrays.
- Rollback: Revert the PR.
- Gates run: targeted `cargo build`, `cargo test`, `cargo fmt`, `cargo clippy`.

## 🗂️ .jules artifacts
- `.jules/runs/sentinel_redaction/envelope.json`
- `.jules/runs/sentinel_redaction/decision.md`
- `.jules/runs/sentinel_redaction/receipts.jsonl`
- `.jules/runs/sentinel_redaction/result.json`
- `.jules/runs/sentinel_redaction/pr_body.md`

## 🔜 Follow-ups
None
