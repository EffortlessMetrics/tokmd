# tokmd Roadmap

This document outlines the path from the baseline (v0.1) to a fully implemented **v1.0** receipt-grade tool.

> **Status (v0.2.0)**: Milestones 1 through 4 have been largely implemented. The focus is now on verification, CI hardening, and final release prep.

## Philosophy
`tokmd` is designed to be a **sensor** that emits **receipts**. It shifts work left, allowing humans and agents to review stable, structured artifacts rather than raw, messy output.

## Milestones

### Milestone 1: Publish Hygiene + CLI Correctness (✅ Completed in v0.2.0)
**Goal:** Make it safe to install and unsurprising.

- [x] **Metadata Polish**: `Cargo.toml` metadata updated.
- [x] **Output Safety**: `format.rs` ensures newlines.
- [x] **Defaults**: `export` defaults to `jsonl`.
- [ ] **Documentation**: 
    - [ ] Document "first wins" file deduplication logic.
    - [ ] Explain embedded language behavior in README.

### Milestone 2: Receipt Schema (✅ Completed in v0.2.0)
**Goal:** Outputs become self-describing "receipts" suitable for pipelines.

- [x] **Structured Envelopes**: `LangReceipt`, `ModuleReceipt`, `ExportMeta` implemented in `format.rs`.
- [x] **Schema Fields**: `schema_version`, `tool`, `mode`, `inputs`, `totals`, `rows` captured.
- [x] **Stability**: `serde` serialization used.

### Milestone 3: Semantic Coherence (✅ Completed in v0.2.0)
**Goal:** Align behavior across `lang`, `module`, and `export` views.

- [x] **Unified Children Flag**: `--children` supported in `module` and `export`.
- [x] **Behaviors**: `parents-only` vs `separate` implemented in `model.rs`.
- [x] **Export Clarity**: `kind` (parent/child) field added to `FileRow`.

### Milestone 4: LLM/Pipeline Ergonomics (✅ Completed in v0.2.0)
**Goal:** Make it safe, cheap, and robust for machine consumption.

- [x] **Filters**: `--min-code` and `--max-rows` implemented.
- [x] **Path Safety**: `redact`, `strip-prefix` implemented.
- [x] **Robustness**: `csv` crate used for export.

## Future (v1.1+)
- [ ] **Bundled Receipt**: `tokmd receipt` command to bundle lang + module + export into one artifact.
- [ ] **GitHub Action**: Official wrapper for CI usage.
- [ ] **Diff Mode**: Comparison between two receipts or git refs.
