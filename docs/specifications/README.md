# tokmd Specifications

This directory captures implementation-facing specifications for tokmd's major surfaces.
Specifications in this folder are intended to be:

- **Normative** where behavior must remain stable for compatibility.
- **Testable** through existing integration/golden/property tests.
- **Version-aware** so schema and CLI changes are explicit.

## Specification Index

| Spec | Status | Scope |
|------|--------|-------|
| [Core Receipt Contract](./core-receipt-contract.md) | Active | `lang`, `module`, `export`, `diff`, `run` envelope + deterministic ordering |

## Writing Rules

1. Use RFC-2119 style keywords (**MUST**, **SHOULD**, **MAY**) for normative requirements.
2. Keep examples concise and stable; avoid timestamps unless required.
3. Every normative section SHOULD map to at least one existing test suite or a test TODO.
4. Any schema-shape change MUST include:
   - schema version update in code,
   - `docs/schema.json` update,
   - `docs/SCHEMA.md` update,
   - changelog note.
5. If a specification introduces a tradeoff or long-lived policy, add/update an ADR in `docs/adrs/`.
