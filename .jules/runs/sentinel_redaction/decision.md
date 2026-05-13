# Decision

## Option A (recommended)
Preserve explicitly known safe compound suffixes such as `.tar.gz`, otherwise keep only the final allowlisted extension. Unsafe final extensions still suppress all suffix output.

- **Structure:** Tightens boundary hardening by strictly adhering to the allowlist.
- **Velocity:** Low risk, minimal code change.
- **Governance:** Improves redaction safety.

## Option B
Use `Path::new().extension()`, which only retrieves the final extension.

- **Trade-offs:** Keeps the old final-extension contract but drops useful compound archive suffix context such as `.tar.gz`.

## Decision
Option A. It preserves semantic archive suffixes like `.tar.gz` without preserving arbitrary safe-looking chains such as `.json.rs`, and it still hides unsafe suffixes like `.rs.bak`.
