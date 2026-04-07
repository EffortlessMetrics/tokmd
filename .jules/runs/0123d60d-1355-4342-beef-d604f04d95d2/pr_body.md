# PR Glass Cockpit

## Type
- [x] Bugfix
- [ ] Feature
- [x] Hardening
- [ ] Refactor
- [ ] Documentation

## Purpose
`tokmd-format`'s `write_export_json` and `write_export_jsonl` failed to redact `module_roots` structural arrays in `ExportArgsMeta` and `ExportData` metadata structures when executing under `RedactMode::All`. This exposed the underlying codebase directory structures to external systems and output traces.

This patch applies boundary hardening to structurally verify and apply `tokmd_redact::short_hash` string modifications to those roots prior to format writing.

## Approach
* Implemented `redact_module_roots` helper utility inside `tokmd-format`.
* Used this helper to obfuscate `.module_roots` inside `ExportMeta`, `ExportReceipt`, and `ExportData` serialization wrappers dynamically if `RedactMode::All` is set.
* Addressed leakage securely directly at the formatting edge boundary, ensuring no unexpected traces leak via output logs.

## Verification

- Confirmed stability via local workspace checks.
