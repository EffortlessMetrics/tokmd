# BDD Traceability Matrix

This matrix links behavior specifications to implementation modules and executable tests.

| Behavior ID | BDD Scenario (Given / When / Then) | Implementation anchors | Primary executable tests |
|---|---|---|---|
| BDD-SCAN-01 | Given mixed separators + ignore rules, when scanning, then normalized deterministic file rows are emitted. | `crates/tokmd-scan/src/path/mod.rs`, `crates/tokmd-scan/src/tokeignore/mod.rs`, `crates/tokmd-scan/src/lib.rs` | `crates/tokmd-scan/src/path/tests.rs`, `crates/tokmd/tests/bdd_scenarios_w75.rs`, `crates/tokmd/tests/determinism_hardening_w51.rs` |
| BDD-RECEIPT-01 | Given any scan result, when receipts are serialized, then schema/version envelope contracts hold. | `crates/tokmd-types`, `crates/tokmd-core/src/lib.rs`, `crates/tokmd-core/src/ffi.rs` | `crates/tokmd/tests/schema_sync.rs`, `crates/tokmd/tests/schema_doc_sync.rs`, `crates/tokmd/tests/receipt_contracts_w72.rs` |
| BDD-FORMAT-01 | Given a receipt, when Markdown/TSV/JSON outputs are requested, then deterministic formatting is produced. | `crates/tokmd-format/src/lib.rs`, `crates/tokmd-format/src/analysis/markdown.rs`, `crates/tokmd-format/src/export_tree/mod.rs` | `crates/tokmd/tests/output_formats_w76.rs`, `crates/tokmd/tests/markdown_output.rs`, `crates/tokmd/tests/json_output.rs` |
| BDD-DIFF-01 | Given two runs/baselines, when diff/cockpit is executed, then reproducible deltas and evidence are produced. | `crates/tokmd-core/src/lib.rs`, `crates/tokmd-core/src/settings.rs`, CLI integration in `crates/tokmd` | `crates/tokmd/tests/bdd_diff_scenarios_w50.rs`, `crates/tokmd/tests/deep_run_cockpit_w52.rs`, `crates/tokmd/tests/diff_deep_w77.rs` |
| BDD-ANALYZE-01 | Given a repository and presets, when analysis runs, then enrichments match preset contracts. | `crates/tokmd-analysis`, `crates/tokmd-format/src/analysis/mod.rs`, `crates/tokmd-core/src/lib.rs` | `crates/tokmd/tests/bdd_analyze_scenarios_w50.rs`, `crates/tokmd/tests/analyze_integration.rs`, `crates/tokmd/tests/full_pipeline_w55.rs` |

## Usage

1. Add or update BDD rows when behavior changes.
2. Ensure each row has at least one executable regression test.
3. Prefer scenario-focused test names (`bdd_*`/`*_integration`) for discoverability.
