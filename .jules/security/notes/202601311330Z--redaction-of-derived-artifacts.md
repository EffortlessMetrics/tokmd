# Redaction of Derived Artifacts

**Context**: When `tokmd run` generates artifacts (`module.json`), it uses internal models (`ModuleReport`) that are computed from raw scan data. Even if `scan` inputs are normalized, the derived model fields (like `module` names) retain original values.

**Pattern**: `export.jsonl` was correctly redacted because it passed through `redact_rows` during serialization. However, `module.json` was serialized directly from the model, bypassing redaction.

**Guidance**:
- Any artifact that outputs user-controlled strings (paths, module names) must have a redaction pass if `RedactMode` is active.
- Do not assume "summary" reports are safe; module names are often directory names which can be sensitive.
- Prefer applying redaction at the "last mile" (formatting/serialization) rather than in the core model, to keep the model clean.

**Prevention**:
- Use `format::redact_*` helpers for all reports before writing.
- Add regression tests that inspect *all* generated artifacts for leaked strings when redaction is on.
