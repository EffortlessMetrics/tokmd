# tokmd Implementation Specification

## Status

Draft v1 (April 29, 2026).

## Purpose

This specification translates product requirements into implementation-level contracts for maintainers and contributors. It is intended to be read alongside:

- `docs/requirements.md` (what must be true)
- `docs/design.md` (why the system is shaped this way)
- `docs/architecture.md` (where behavior lives)
- `docs/SCHEMA.md` + `docs/schema.json` (what is serialized)

## Scope

This document defines implementation contracts for:

1. Scan and aggregation workflows (`lang`, `module`, `export`, `run`, `diff`)
2. Analysis orchestration and presets (`analyze` and enrichers)
3. Deterministic output behavior and ordering guarantees
4. Schema versioning and compatibility rules
5. Runtime-facing interfaces (CLI, FFI, Python, Node)

It does **not** redefine every CLI option; see `docs/reference-cli.md` for flag details.

---

## 1. System Invariants

### 1.1 Determinism Invariant

For a fixed repository state, configuration, and feature set, tokmd outputs must be stable across repeated executions on the same platform and equivalent across platforms after path normalization.

Implementation obligations:

- Use deterministic key/value containers in output paths (prefer `BTreeMap` over `HashMap` in receipt-facing structures).
- Apply explicit sort ordering to row collections:
  - primary: descending `code`
  - secondary: ascending normalized `name`
- Normalize paths to forward-slash form before formatting and serialization.

### 1.2 Envelope Invariant

JSON receipt families must include envelope metadata with schema identifiers and version constants from the relevant `*-types` crate.

Implementation obligations:

- Do not introduce JSON shape changes without incrementing the corresponding schema constant.
- Keep schema docs synchronized with code changes (`docs/SCHEMA.md`, `docs/schema.json`, and any family-specific schema docs).

### 1.3 Tier Boundary Invariant

Tier-5 crates (`tokmd`, bindings) must consume analysis orchestration via Tier-4 facades instead of importing Tier-3 formatting/orchestration crates directly where a facade exists.

Implementation obligations:

- Keep analysis formatting calls routed through `tokmd-core::analysis_facade`.
- Maintain architecture constraints documented in ADR-001.

---

## 2. Workflow Contracts

### 2.1 Scan Contract

Input: scan settings + include/exclude controls.  
Output: normalized inventory of language/file/module-level counters.

Required behavior:

- Respect ignore rules and explicit include/exclude patterns.
- Support children handling modes:
  - `Collapse`: embedded/child language stats roll into parent totals.
  - `Separate`: embedded contributions appear as explicit `(embedded)` rows.

### 2.2 Modeling Contract

Input: scan inventory.  
Output: grouped rows (language/module/file) with totals and percentages.

Required behavior:

- Percentage math must be derived from canonical total code lines.
- Grouping keys must operate on normalized paths.
- Sorting and tie-breaking must be deterministic (see 1.1).

### 2.3 Formatting Contract

Input: typed receipts from model/analysis layers.  
Output: Markdown/TSV/JSON/JSONL/CSV rendering.

Required behavior:

- No formatter is allowed to mutate semantic payload values.
- Human-readable outputs must preserve deterministic order from receipts.
- Machine-readable outputs must preserve envelope metadata and schema versions.

### 2.4 Analysis Contract

Input: baseline receipts plus enabled enrichers/preset.  
Output: analysis receipt with derived metrics and optional enrichments.

Required behavior:

- Presets are additive bundles; each preset must map to a documented feature set.
- Missing optional enrichers (e.g., feature-gated or environment-limited sources) must degrade gracefully with explicit omission rather than invalid placeholder metrics.
- Analysis schema evolution is independent from core receipt schema evolution.

### 2.5 Diff Contract

Input: two receipts/runs or git references.  
Output: directional delta summaries.

Required behavior:

- Use two-dot range semantics (`A..B`) for release/tag comparison behavior where range syntax is used.
- Preserve per-row stability and deterministic ordering in delta output.

---

## 3. Interface Contracts

### 3.1 CLI Contract

- CLI commands are thin orchestration surfaces over typed workflows.
- User-facing behavior must remain backward compatible unless a documented breaking release is planned.
- Help and reference docs must be updated with option changes.

### 3.2 FFI Contract (`tokmd-core`)

- `ffi::run_json(mode, args_json)` is the stable polyglot boundary.
- Responses must be envelope-shaped (`ok`, `data`/`error`).
- Error payloads must be serializable and structured (not ad-hoc plaintext only).

### 3.3 Python/Node Bindings Contract

- Bindings expose the same semantic modes as Rust workflows/FFI.
- Long-running operations must avoid blocking host runtime critical paths (GIL release / blocking task offload).
- Binding-level API changes must be documented in release notes and migration guidance.

---

## 4. Quality Gates

Minimum pre-merge validation for implementation changes should include:

1. Formatting check (`cargo fmt-check`)
2. Lint check (`cargo clippy -- -D warnings` or repo gate equivalent)
3. Relevant tests (`cargo test --verbose` or targeted crate/test scope)
4. Schema sync checks when JSON shape changes are introduced

For output-affecting changes, update golden snapshots where applicable.

---

## 5. Change Control

A change requires an ADR when it modifies one or more of the following:

- Tier boundaries or crate dependency direction
- Determinism/ordering rules
- Serialization envelope semantics or schema version policy
- Runtime interface contracts (CLI compatibility policy, FFI envelope model)

Recommended ADR template fields:

- Context
- Decision
- Consequences (positive/negative)
- Rollout / migration notes

See `docs/adr/` for accepted ADRs.
