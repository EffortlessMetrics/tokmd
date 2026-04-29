# tokmd Specification

## Purpose

This specification defines the implementation contract for `tokmd` as both a CLI and an embeddable analysis engine. It is intended to be implementation-facing (what the system **must** do) and pairs with architecture and schema docs.

## Scope

This document covers:

- repository scan and inventory behavior
- receipt families and schema compatibility
- determinism and reproducibility requirements
- command behavior and artifact guarantees
- embedding contracts (Rust, FFI, Python, Node)

This document does not replace user-focused tutorials or CLI flag reference.

## Normative language

The keywords **MUST**, **SHOULD**, and **MAY** are normative.

## 1) System model

`tokmd` is a single-scan, multi-view system:

1. Collect source inventory from repository paths.
2. Aggregate into normalized receipt models.
3. Render outputs in human and machine formats.
4. Optionally enrich with analysis presets and adapters.

Implementations MUST preserve the tiered crate boundaries documented in `docs/architecture.md`.

## 2) Receipt-first contract

### 2.1 Receipt families

Implementations MUST maintain separate schema-version lines for independent receipt families:

- Core receipts (`lang`, `module`, `export`, `diff`, `run`)
- Analysis receipts
- Cockpit receipts
- Handoff manifests
- Context receipts
- Context bundle receipts

### 2.2 Schema change policy

When the JSON structure of a receipt family changes:

1. The corresponding schema version constant MUST be incremented.
2. `docs/schema.json` and/or family-specific schema docs MUST be updated.
3. Golden snapshots and integration tests MUST be refreshed.

## 3) Determinism requirements

For identical input trees and settings, implementations MUST produce byte-stable machine outputs.

Determinism depends on:

- ordered map/set usage (`BTree*`) at output boundaries
- stable sorting (descending by code lines, ascending by name tie-break)
- normalized slash-separated paths
- normalized test timestamps where snapshots are used

Any new feature that emits rows or keys MUST explicitly define stable ordering.

## 4) Scan and aggregation behavior

### 4.1 Scan pass

A single inventory scan SHOULD be reused across downstream views where possible.

### 4.2 Embedded language policy

Implementations MUST support both children modes:

- Collapse: embedded children roll into parent totals
- Separate: embedded children appear as explicit rows

The selected mode MUST be applied consistently across all relevant commands.

### 4.3 Path behavior

All outward-facing paths MUST be normalized to `/` regardless of host OS.

## 5) Command-level guarantees

### 5.1 Inventory commands

- `tokmd lang` MUST produce language totals and totals metadata.
- `tokmd module` MUST group by normalized module key.
- `tokmd export` MUST emit file-level inventory in requested format.

### 5.2 Orchestration commands

- `tokmd run` MUST support artifact output for repeatable workflows.
- `tokmd analyze` MUST support preset-driven enrichment and explicit feature gating.
- `tokmd diff` MUST compare receipts/runs with explicit range inputs.
- `tokmd cockpit` and `tokmd gate` MUST return policy-relevant evidence in machine-readable form.

## 6) Embedding and FFI

### 6.1 Rust facade

`tokmd-core` workflow functions MUST remain clap-free and settings-driven.

### 6.2 JSON FFI envelope

`ffi::run_json(mode, args_json)` MUST return an envelope with:

- `ok: bool`
- `data` on success
- `error` on failure

Binding layers (Python/Node/WASM) SHOULD map this envelope without lossy transformations.

## 7) Optional adapters and feature flags

Adapters that require external context (Git history, file content inspection, filesystem walk variants) MUST remain behind explicit feature flags and degrade gracefully when disabled.

## 8) Error and exit semantics

CLI implementations MUST preserve distinct outcome classes:

- success with full receipt
- runtime/tooling error
- policy gate failure
- skip/missing optional input where applicable

Exit-code semantics MUST remain stable across minor releases unless explicitly versioned and documented.

## 9) Verification requirements

Before merging behavior changes, contributors SHOULD run:

- formatting checks
- unit/integration tests
- snapshot verification for changed receipts
- strict linting when practical (`clippy -D warnings`)

## 10) Traceability to ADRs

Major architectural decisions that alter boundaries, contracts, or determinism MUST be captured under `docs/adr/` and referenced from architecture/specification docs.
