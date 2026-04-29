# Implementation Specifications

This document defines implementation-level specifications for the core `tokmd` workflows.

## Purpose

- Establish testable, behavior-oriented contracts for CLI and receipt outputs.
- Provide traceability between product behavior, BDD scenarios, and architecture decisions.
- Reduce ambiguity for future feature work and refactors.

## Spec Index

| Spec ID | Area | Source of Truth | BDD Coverage |
|---|---|---|---|
| `SPEC-LANG-001` | `tokmd lang` summary behavior | `tokmd` CLI + `tokmd-model` sorting | `crates/tokmd/tests/bdd_lang_scenarios_w50.rs`, `crates/tokmd/tests/bdd_scenarios_w75.rs` |
| `SPEC-MODULE-001` | `tokmd module` aggregation behavior | module depth + normalized module keys | `crates/tokmd/tests/bdd_module_scenarios_w50.rs`, `crates/tokmd/tests/bdd_scenarios_w75.rs` |
| `SPEC-EXPORT-001` | `tokmd export` line-oriented inventory contracts | JSONL/CSV export rows + metadata | `crates/tokmd/tests/bdd_export_scenarios_w50.rs`, `crates/tokmd/tests/bdd_scenarios_w75.rs` |
| `SPEC-ANALYZE-001` | `tokmd analyze` enrichment contract | preset-driven analysis pipeline | `crates/tokmd/tests/bdd_analyze_scenarios_w50.rs` |
| `SPEC-DIFF-001` | `tokmd diff` change-analysis contract | receipt diff semantics | `crates/tokmd/tests/bdd_diff_scenarios_w50.rs` |

## Behavioral Requirements

### `SPEC-LANG-001`
- Given a repository with supported source files, when `tokmd lang --format json` runs, then rows MUST include each detected language.
- Language rows MUST be sorted by descending `code` and then by language name for deterministic output.

### `SPEC-MODULE-001`
- Given nested paths, when `tokmd module --module-depth N` runs, then module keys MUST reflect normalized path separators.
- Module output MUST remain stable across platforms via path normalization.

### `SPEC-EXPORT-001`
- Given one or more files, when `tokmd export --format jsonl` runs, then output MUST include valid JSON lines.
- Data rows MUST include file identity (`path`) and sizing fields (`code`, language).

### `SPEC-ANALYZE-001`
- Given an input receipt or direct scan target, when `tokmd analyze` runs with a preset, then output MUST include deterministic, schema-versioned analysis artifacts.

### `SPEC-DIFF-001`
- Given baseline and target receipts/refs, when `tokmd diff` runs, then deltas MUST reflect net additions/removals and preserve deterministic ordering in rendered output.

## Traceability Conventions

- BDD tests should keep `Given/When/Then` names and map to one or more `SPEC-*` IDs in module docs.
- ADRs under `docs/adrs/` capture rationale for long-lived implementation choices used by these specs.
