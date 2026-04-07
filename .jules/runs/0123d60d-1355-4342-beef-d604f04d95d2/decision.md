# Sentinel Decision: Redacting Module Roots under RedactMode::All

## Context
The Sentinel persona focuses on redaction correctness and leakage prevention. While analyzing the `core-pipeline` shard, specifically how the `RedactMode::All` operates, it was discovered that `tokmd-format` fails to redact the `module_roots` array structure inside the metadata (`ExportArgsMeta` and `ExportData`) objects for JSON and JSONL exports.

## Options
### Option A: Do Nothing / Treat as Out of Scope
Leave `module_roots` clear, assuming structural leakage is acceptable if paths are obfuscated.

### Option B: Implement Redaction for Module Roots
When `args.redact == RedactMode::All`, properly apply `tokmd_redact::short_hash` on `module_roots` elements inside the formatters (`write_export_json` and `write_export_jsonl`), preventing structural layout leakage.

## Decision
**Option B is chosen.** Consistent with Sentinel's focus on leakage prevention, any field that might betray file tree structure details under `RedactMode::All` should be obfuscated securely. A `redact_module_roots` helper function was injected to selectively run `tokmd_redact::short_hash` on module arrays within format exports.
