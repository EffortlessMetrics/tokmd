# Specification-by-Example and BDD Traceability

This document defines how tokmd implementation behavior is specified, traced to code, and verified through BDD scenarios.

## Goals

- Keep behavior contracts human-readable and executable.
- Link user-facing behavior to owning modules and integration tests.
- Make regressions easy to triage by preserving scenario-level intent.

## BDD Workflow

1. **Specify behavior** in `Given / When / Then` form.
2. **Map the behavior** to an owning workflow or module.
3. **Implement/adjust code** at the owner boundary.
4. **Add or update a BDD scenario test** in `crates/tokmd/tests/`.
5. **Preserve traceability** by keeping scenario IDs stable.

## Scenario Registry

| Scenario ID | Behavior Contract | Primary Code Owner | BDD/Integration Tests |
|---|---|---|---|
| LANG-001 | Language totals are deterministic and ordered by code desc, name asc | `tokmd-model`, `tokmd-format` | `crates/tokmd/tests/bdd_lang_scenarios_w50.rs` |
| MODULE-001 | Module keys derive from normalized slash paths | `tokmd-model` | `crates/tokmd/tests/bdd_module_scenarios_w50.rs` |
| EXPORT-001 | Export emits stable file-level inventory contracts | `tokmd-format`, `tokmd-core` | `crates/tokmd/tests/bdd_export_scenarios_w50.rs` |
| ANALYZE-001 | Preset-driven analysis enrichments remain contract-stable | `tokmd-analysis`, `tokmd-analysis-types` | `crates/tokmd/tests/bdd_analyze_scenarios_w50.rs`, `crates/tokmd/tests/analyze_integration.rs` |
| DIFF-001 | Diff compares receipts/runs with deterministic reporting | `tokmd-core`, `tokmd-format` | `crates/tokmd/tests/bdd_diff_scenarios_w50.rs`, `crates/tokmd/tests/run_diff.rs` |
| CLI-001 | End-to-end user workflows remain behaviorally stable | `crates/tokmd/src` command surface | `crates/tokmd/tests/bdd_scenarios_w71.rs`, `crates/tokmd/tests/bdd_scenarios_w75.rs` |

## Spec Authoring Rules

- Prefer behavior language over implementation details.
- Every new user-visible behavior must reference at least one scenario ID.
- Every scenario ID must map to at least one automated test file.
- When behavior changes, update this registry and test comments in the same PR.

## Definition of Done for Behavioral Changes

A change is complete only when:

- A behavior contract is documented in this file.
- A code owner is identified.
- A BDD or integration test asserts the contract.
- Any affected architecture decision is captured under `docs/adrs/`.

## Related Documents

- `docs/requirements.md` for product-level requirements.
- `docs/architecture.md` for crate boundaries and dependency constraints.
- `docs/testing.md` for test pyramid and execution guidance.
- `docs/adrs/0001-tier-facade-and-bdd-traceability.md` for decision rationale.
