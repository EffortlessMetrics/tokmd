# Receipt Pipeline Specification

## Status
Accepted

## Scope
Defines invariants and interfaces for the core inventory receipt path used by:

- `tokmd lang`
- `tokmd module`
- `tokmd export`
- `tokmd run`

## Pipeline Stages

1. **Scan** (`tokmd-scan`)
   - Input: repository root, include/exclude filters, child-language mode.
   - Output: normalized file/language counts.
2. **Model** (`tokmd-model`)
   - Input: scan aggregates.
   - Output: language/module/file rows with deterministic ordering.
3. **Format** (`tokmd-format`)
   - Input: model receipts.
   - Output: Markdown/TSV/JSON/JSONL/CSV representations.
4. **Delivery** (`tokmd`, `tokmd-core`)
   - Output adapters: CLI stdout/files, FFI JSON envelopes, language bindings.

## Required Invariants

- Paths MUST be normalized to forward slashes (`/`) prior to any keying or output.
- Ordered output MUST be deterministic:
  - sort descending by `code` lines,
  - then ascending by stable name key.
- Embedded language behavior MUST honor `ChildrenMode` consistently:
  - `Collapse`: merge into parent totals,
  - `Separate`: emit explicit embedded rows.
- Core receipt JSON outputs MUST include envelope metadata with `schema_version = 2`.

## Error Semantics

- Scan-stage file permission/decoding failures SHOULD be represented as structured warnings when recoverable.
- Irrecoverable configuration or I/O failures MUST produce a non-zero command exit (CLI) or `ok=false` envelope (FFI).

## Test Expectations

- Golden snapshot coverage for representative repos and child-language fixtures.
- Property tests for ordering and aggregation invariants.
- Cross-output parity tests ensuring JSON/Markdown/TSV totals align for the same run.
