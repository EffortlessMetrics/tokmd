# Core Receipt Contract Specification

- **Status:** Active
- **Last updated:** 2026-04-29
- **Applies to:** core receipts (`lang`, `module`, `export`, `diff`, `run`)
- **Primary crates:** `tokmd-types`, `tokmd-format`, `tokmd-core`, `tokmd`

## 1. Envelope Requirements

1. Core JSON receipts **MUST** include an envelope containing `schema_version`.
2. Core receipt schema version **MUST** match `SCHEMA_VERSION = 2` until a breaking shape change is introduced.
3. Producers **MUST NOT** omit envelope metadata when JSON output mode is selected.

## 2. Deterministic Ordering

1. Aggregation maps exposed in receipts **MUST** be deterministically ordered.
2. Stable key ordering **MUST** use `BTreeMap`-equivalent behavior, not hash-randomized maps.
3. Human-readable ordered rows **MUST** sort by:
   1. descending `code` lines,
   2. ascending name/path tie-break.

## 3. Path Semantics

1. Paths emitted in machine-readable or human-readable receipts **MUST** be normalized to forward slashes (`/`).
2. Module keys **MUST** be derived from normalized paths.
3. Platform-specific separators **MUST NOT** leak into emitted receipt payloads.

## 4. Embedded Language Handling

1. `ChildrenMode::Collapse` **MUST** merge embedded-language counts into parent language totals.
2. `ChildrenMode::Separate` **MUST** emit separate embedded rows labeled as embedded.
3. The selected children mode **MUST** be applied consistently for all receipt families where supported.

## 5. Compatibility and Change Management

1. Any JSON shape change in core receipts **MUST** increment the core schema version and update documentation artifacts.
2. Backward-incompatible CLI output behavior changes **SHOULD** be called out in release notes.
3. New fields **SHOULD** be additive where possible to preserve consumer compatibility.

## 6. Verification Matrix

- Integration tests in `crates/tokmd/tests/` validate CLI-level receipt behavior.
- Golden snapshots verify stable ordering and formatting.
- Crate-level tests in `tokmd-types` and `tokmd-format` validate shape and rendering invariants.

## 7. Non-goals

- Defining analysis receipt semantics (covered by analysis-specific contracts).
- Defining cockpit gate policy semantics.
- Defining transport-level API guarantees outside `tokmd-core`/CLI artifacts.
