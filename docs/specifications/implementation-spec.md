# Implementation Specification

## Scope

This specification defines implementation contracts for tokmd's core workflows and outputs, with emphasis on deterministic receipts, schema governance, and CLI/API parity.

## Contract Areas

### 1) Scan and Inventory
- The scan layer MUST normalize all paths to `/` separators before model aggregation.
- Embedded language accounting MUST follow a single selectable policy (`collapse` or `separate`) for each run.
- File inclusion MUST honor `.gitignore` and `.tokeignore` unless explicitly overridden.

### 2) Modeling and Ordering
- Aggregations MUST use deterministic map/set structures.
- Sort precedence MUST be: descending code lines, then ascending stable name key.
- Null/empty result sets MUST still serialize valid envelopes with schema metadata.

### 3) Output Receipts
- JSON-family outputs MUST include schema version metadata and generation context.
- CSV/TSV headers MUST remain stable and additive-only inside a schema major.
- Redacted outputs MUST preserve deterministic hashing behavior for the same inputs.

### 4) CLI, Core, and Bindings Parity
- CLI command modes and `tokmd-core` workflow entry points MUST represent equivalent behaviors.
- FFI `run_json` modes MUST preserve response envelope shape (`ok`, `data`, `error`).
- Python and Node bindings MUST not mutate domain semantics relative to core workflows.

### 5) Policy and Gates
- `gate` evaluation MUST support explicit failure reasons and machine-readable JSON pointers.
- Missing optional evidence MUST produce "skipped/unknown" semantics, not silent pass.
- Exit code semantics MUST match requirements: `0` success, `1` runtime/tool error, `2` policy failure.

## Acceptance Criteria

A change is implementation-complete when:
1. Behavior is represented by BDD scenarios in `docs/bdd.md`.
2. Schema-impacting changes include schema/version notes.
3. Determinism is validated by snapshots/property tests where applicable.
4. Any architectural tradeoff is captured by a linked ADR in `docs/adr/`.
