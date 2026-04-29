# Core Workflow Specification (BDD)

This specification defines minimum contract behavior for `lang`, `module`, `export`, and `diff` workflows through both CLI and core library interfaces.

## Invariants

1. Output ordering must be deterministic for equivalent inputs.
2. Paths emitted in receipts must be normalized for cross-platform parity.
3. JSON interfaces must return schema-tagged envelopes.
4. CLI and core workflows should preserve behavioral parity for shared modes.

## Scenario: language summary via CLI

**Given** a repository with supported source files  
**When** a user runs `tokmd lang --format json`  
**Then** the command returns a stable JSON receipt with deterministic language rows.

Implementation links:
- CLI command handler: `crates/tokmd/src/commands/lang.rs`
- Parser wiring: `crates/tokmd/src/cli/parser.rs`

Validation links:
- `crates/tokmd/tests/bdd_lang_scenarios_w50.rs`
- `crates/tokmd/tests/cli_snapshot_golden.rs`
- `crates/tokmd/tests/determinism.rs`

## Scenario: module breakdown via core workflow

**Given** normalized scan input  
**When** `module_workflow` runs in `tokmd-core`  
**Then** it returns module-grouped output that follows deterministic sort behavior.

Implementation links:
- Core facade: `crates/tokmd-core/src/lib.rs`
- Settings and mode wiring: `crates/tokmd-core/src/settings.rs`

Validation links:
- `crates/tokmd-core/tests/workflows.rs`
- `crates/tokmd/tests/bdd_module_scenarios_w50.rs`
- `crates/tokmd/tests/determinism_hardening.rs`

## Scenario: export file inventory

**Given** a mixed-language repository  
**When** a user runs `tokmd export` in JSON/JSONL/CSV variants  
**Then** each format reflects the same inventory semantics.

Implementation links:
- Export command: `crates/tokmd/src/commands/export.rs`
- Output bundling helpers: `crates/tokmd/src/export_bundle.rs`

Validation links:
- `crates/tokmd/tests/bdd_export_scenarios_w50.rs`
- `crates/tokmd/tests/json_output.rs`
- `crates/tokmd/tests/cli_output_formats.rs`

## Scenario: diff contract across tags or receipts

**Given** two comparable scans or receipt inputs  
**When** `tokmd diff` runs with release-style comparisons  
**Then** it reports meaningful deltas under the documented range semantics.

Implementation links:
- Diff command: `crates/tokmd/src/commands/diff.rs`
- Git range support: `crates/tokmd/src/git_support.rs`

Validation links:
- `crates/tokmd/tests/bdd_diff_scenarios_w50.rs`
- `crates/tokmd/tests/diff_w71.rs`
- `crates/tokmd/tests/run_diff.rs`

## Scenario: JSON FFI envelope stability

**Given** a consumer calling `run_json(mode, args_json)`  
**When** a valid mode is requested  
**Then** response is wrapped in `{ ok, data | error }` envelope and preserves schema contract.

Implementation links:
- FFI entrypoint: `crates/tokmd-core/src/ffi.rs`
- Error mapping: `crates/tokmd-core/src/error.rs`

Validation links:
- `crates/tokmd-core/tests/ffi_contract.rs`
- `crates/tokmd-core/tests/json_api.rs`
- `crates/tokmd-core/tests/bindings_parity.rs`
