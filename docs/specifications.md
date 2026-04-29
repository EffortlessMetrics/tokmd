# tokmd Implementation Specifications

This document defines implementation-level specifications for core tokmd execution paths.

## 1. Scan and Inventory Specification

### Inputs
- Repository root path.
- `ScanOptions` (`exclude`, `include_hidden`, `no_ignore`, `children_mode`, and feature toggles).

### Processing Contract
1. Normalize all candidate paths to forward slashes before any keying/output.
2. Execute one scan pass for language detection/counting.
3. Aggregate via model layer into language, module, and file rows.
4. Sort rows by code desc, name asc.
5. Emit deterministic receipts using ordered collections only.

### Outputs
- `LangReceipt`, `ModuleReceipt`, `ExportReceipt`, or combined `RunReceipt` envelope.
- Metadata with schema versions from `tokmd-types`.

### Non-Functional Requirements
- Deterministic for identical inputs.
- No clap dependency leakage below product tier.
- Optional adapters (`git`, `walk`, `content`) must remain feature-gated.

## 2. Analysis Orchestration Specification

### Inputs
- Base inventory receipt and/or export dataset.
- Preset selection (`receipt`, `health`, `risk`, `supply`, `architecture`, `topics`, `security`, `identity`, `git`, `deep`, `fun`).

### Processing Contract
1. Resolve preset to enrichers in stable order.
2. Run deterministic enrichers first (derived metrics/grid).
3. Run optional enrichers only when required inputs/features exist.
4. Represent missing inputs as `skip` states instead of hard failures where possible.
5. Render outputs through `tokmd-format::analysis` only.

### Outputs
- Analysis receipt with `ANALYSIS_SCHEMA_VERSION`.
- Optional markdown/json/svg/html representations.

### Non-Functional Requirements
- Preserve stable key ordering and sorting.
- Maintain explicit provenance in receipts for skipped enrichers.

## 3. Facade and Binding Specification

### Rust Facade
- `tokmd-core` is the only Tier 4 ingress for Tier 5 products.
- Workflow APIs remain clap-free and settings-driven.

### FFI Contract
- `ffi::run_json(mode, args_json) -> String` envelope:
  - success: `{ "ok": true, "data": ... }`
  - error: `{ "ok": false, "error": ... }`
- Modes must remain additive/backward compatible where possible.

### Bindings
- Python: return native dictionaries; release GIL during heavy operations.
- Node: async Promise API backed by blocking-task isolation.
- WASM: serialized outputs only through defined wasm façade surface.

## 4. Schema Governance Specification

1. Update receipt family schema version constants when structure changes.
2. Update `docs/schema.json` and/or family-specific schema docs alongside code.
3. Keep migration notes in `CHANGELOG.md` for breaking schema changes.
4. Snapshot tests must be updated intentionally (no incidental churn).

## 5. Validation Matrix

Minimum validation for implementation changes:
- `cargo fmt-check`
- `cargo test --verbose`
- `cargo clippy -- -D warnings`

Optional validation when touching relevant surfaces:
- `cargo test -p tokmd-scan properties`
- `cargo +nightly fuzz list`
- `cargo mutants --file <path>`
