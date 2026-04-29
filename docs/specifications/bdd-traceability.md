# BDD Traceability Matrix

This matrix links behavior statements to implementation and tests.

| ID | Behavior (Given/When/Then summary) | Implementation | Tests |
|---|---|---|---|
| BDD-LANG-001 | Given source files, when `lang` runs, then deterministic language receipt is produced. | `crates/tokmd/src/commands/lang.rs` | `crates/tokmd/tests/bdd_lang_scenarios_w50.rs`, `crates/tokmd/tests/cli_snapshot_golden.rs` |
| BDD-MOD-001 | Given normalized paths, when module workflow runs, then module ordering and grouping are stable. | `crates/tokmd-core/src/lib.rs` | `crates/tokmd-core/tests/workflows.rs`, `crates/tokmd/tests/bdd_module_scenarios_w50.rs` |
| BDD-EXP-001 | Given repository inventory, when export renders JSON/JSONL/CSV, then semantics remain equivalent across formats. | `crates/tokmd/src/commands/export.rs`, `crates/tokmd/src/export_bundle.rs` | `crates/tokmd/tests/bdd_export_scenarios_w50.rs`, `crates/tokmd/tests/cli_output_formats.rs` |
| BDD-DIFF-001 | Given baseline and candidate receipts, when diff executes, then deltas are reported with documented range rules. | `crates/tokmd/src/commands/diff.rs`, `crates/tokmd/src/git_support.rs` | `crates/tokmd/tests/bdd_diff_scenarios_w50.rs`, `crates/tokmd/tests/diff_w71.rs` |
| BDD-FFI-001 | Given `run_json` request, when mode is valid, then response envelope is stable and mode-consistent. | `crates/tokmd-core/src/ffi.rs` | `crates/tokmd-core/tests/ffi_contract.rs`, `crates/tokmd-core/tests/json_api.rs` |

## Maintenance rule

Whenever behavior changes:

1. Update the relevant scenario text in `core-workflows.md`.
2. Update this matrix row(s).
3. Add or adjust at least one corresponding automated test.
4. If architectural rationale changed, add or supersede an ADR.
