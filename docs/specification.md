# tokmd Specification

## Purpose

This document defines the normative behavior of tokmd across its CLI, library facade, and receipt outputs. It is intended to be stable enough for implementers, integrators, and binding maintainers.

Normative keywords **MUST**, **SHOULD**, and **MAY** follow RFC 2119 semantics.

## 1. Product Scope

`tokmd` is a deterministic repository inventory and analysis system with:

- CLI workflows (`tokmd`, `tokmd lang`, `module`, `export`, `run`, `analyze`, `diff`, `cockpit`, `gate`, etc.)
- Library workflows (`tokmd-core`) for embedding.
- FFI/SDK bindings (Python and Node) with receipt-compatible output.

The system MUST produce machine-readable receipts and human-readable summaries from the same canonical scan/model pipeline.

## 2. Layering and Dependency Rules

The crate architecture is tiered:

`types -> scan -> model -> format -> analysis -> cli`

Requirements:

1. Lower tiers MUST NOT depend on higher tiers.
2. Tier-0 contracts MUST remain serialization-stable unless the relevant schema version is incremented.
3. Tier-4 (`tokmd-core`) MUST provide clap-free embedding APIs.
4. Tier-5 product crates MAY add UX-specific behavior but MUST preserve receipt semantics.

## 3. Determinism

tokmd outputs are deterministic by contract.

Requirements:

1. Keyed collections in receipt emission paths MUST preserve stable order (e.g., `BTreeMap`-style ordering).
2. Ranked rows MUST sort by descending code lines, then by name for tie-breaking.
3. Equal input corpus + equal options + equal version MUST yield byte-stable JSON/JSONL/CSV/Markdown outputs (excluding explicitly timestamped metadata fields where applicable).

## 4. Path Semantics

Requirements:

1. Output paths MUST be normalized to forward slashes (`/`) on all operating systems.
2. Module key computation MUST use normalized paths.
3. Ignore/exclude decisions SHOULD report normalized paths in diagnostics.

## 5. Embedded-Language Semantics

tokmd supports two children modes:

- `Collapse`: embedded languages are merged into the parent language totals.
- `Separate`: embedded rows are emitted explicitly as embedded language entries.

Requirements:

1. Children-mode behavior MUST be consistent across `lang`, `module`, `export`, and `run` flows.
2. Mode selection SHOULD be reflected in human-readable renderers to avoid ambiguity.

## 6. Receipt Contracts and Schema Versioning

Receipt families version independently.

Current schema constants:

- Core receipts: `SCHEMA_VERSION = 2`
- Analysis receipts: `ANALYSIS_SCHEMA_VERSION = 9`
- Cockpit receipts: `COCKPIT_SCHEMA_VERSION = 3`
- Handoff manifests: `HANDOFF_SCHEMA_VERSION = 5`
- Context receipts: `CONTEXT_SCHEMA_VERSION = 4`
- Context bundles: `CONTEXT_BUNDLE_SCHEMA_VERSION = 2`

Requirements:

1. Any breaking JSON structure change MUST increment the matching family schema version.
2. Schema changes MUST update formal schema documentation (`docs/schema.json` and family-specific schema docs/files).
3. Receipts MUST include envelope metadata sufficient for consumers to detect schema family and version.

## 7. Git Range Semantics

Requirements:

1. `A..B` MUST be used for release/tag comparison style flows (e.g., cockpit/diff release deltas).
2. `A...B` SHOULD be reserved for CI branch-divergence workflows.
3. Commands MUST document the active interpretation when accepting user-provided ranges.

## 8. Library and Binding Contracts

### 8.1 `tokmd-core`

- Must expose clap-free workflow functions for lang/module/export/diff semantics.
- Must provide JSON FFI entrypoint behavior compatible with receipt envelopes.

### 8.2 Python Binding

- Must return Python-native structures.
- Should release GIL during long-running scans.

### 8.3 Node Binding

- Must expose async APIs (Promises).
- Should move blocking work off event loop threads.

## 9. Quality Gates

Minimum validation before merge SHOULD include:

1. Formatting check.
2. Workspace tests relevant to touched receipt surfaces.
3. Snapshot updates/reviews where deterministic outputs changed.
4. Clippy/lint checks for touched crates.

## 10. Backward Compatibility Guidance

1. New optional fields SHOULD be additive and default-safe.
2. Existing fields SHOULD NOT change type/meaning without schema bump.
3. Deprecated fields MAY remain for one minor release cycle with explicit migration notes.

## 11. References

- `docs/architecture.md`
- `docs/design.md`
- `docs/requirements.md`
- `docs/SCHEMA.md`
- `docs/schema.json`
