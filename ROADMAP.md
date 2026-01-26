# tokmd Roadmap: Path to v1.0

This document outlines the strategic plan to evolve `tokmd` from its current feature-complete state (v0.2.0) to a hardened, production-ready release (v1.0.0).

## 🎯 Vision for v1.0

At v1.0, `tokmd` is not just a CLI tool; it is a **stable system component**.
- **Receipt-Grade**: Outputs are deterministic, versioned, and safe for automated pipelines.
- **Trustworthy**: Changes to output format are detected by golden tests.
- **Safe**: It prevents information leaks (redaction) and context overflows (limits) by default.

---

## 📊 Status Summary

| Version | Status | Focus |
| :--- | :--- | :--- |
| **v0.1.0** | ✅ Complete | Basic functionality (scan -> model -> format). |
| **v0.2.0** | ✅ Complete | Feature complete: Receipt schema, Filters, Redaction, Export logic. |
| **v0.9.0** | ✅ Complete | Assurance: Integration tests, Golden snapshots, Edge case verification. |
| **v1.0.0** | 🚀 RC Ready | Stability: Frozen schema, Diataxis docs, Release automation. |

---

## 🛠️ Remaining Workstreams

### 1. Core Confidence (Testing & Determinism)
*Current State: ✅ Complete (Integration tests & Golden snapshots implemented).*

### 2. The Schema Contract
*Current State: ✅ Complete (Formal JSON Schema in `docs/schema.json`).*

### 3. Developer Experience (DX)
*Current State: ✅ Complete (Diataxis docs, Recipes, CLI Help).*

---

## 📅 Detailed Milestone Plan

### ✅ Milestone 1–4 (Completed in v0.2.0)
- **Hygiene**: Metadata, cleanups.
- **Schema**: `LangReceipt`, `ModuleReceipt`, `ExportMeta` structs implemented.
- **Semantics**: Unified `--children` flag, consistent module aggregation.
- **Ergonomics**: `--min-code`, `--max-rows`, `--redact`, `--strip-prefix` implemented.

### ✅ Milestone 5: The Test Harness (v0.9.0)
**Goal**: Refactoring is safe; Output is frozen.

- [x] **Infrastructure**: Add `dev-dependencies` (`assert_cmd`, `predicates`, `tempfile`).
- [x] **Golden Tests**: Snapshot the JSONL output using `insta`.
- [x] **Path Normalization**: Ensure `\` vs `/` doesn't affect sorting.
- [x] **Redaction Verification**: Verify `tokmd export --redact all` leaks no PII.

### ✅ Milestone 6: Documentation & Polish (v0.9.5)
**Goal**: Users understand the "Receipt" concept.

- [x] **Recipe Book**: Add `docs/recipes.md`.
- [x] **Schema Docs**: Document the fields of `LangReceipt` and `ExportRow`.
- [x] **Formal Schema**: JSON Schema Draft 07 in `docs/schema.json`.
- [x] **Diataxis Structure**: Tutorial, How-to, Reference, Explanation.

### 🚀 Milestone 7: v1.0.0 Launch
**Goal**: Stability.

- [x] **SemVer Lock**: Schema is frozen.
- [x] **Release Automation**: GitHub Action for binary releases.
- [x] **Crates.io**: Final publish (ready for user action).

---

## 🔭 Future Horizons (v1.x+)

These features extend `tokmd` from a passive sensor to an active agent in the LLM workflow.

### A. The "Context Budget" Features
*Goal: Help users fit code into context windows.*

- **Token Estimation**: Add an `estimated_tokens` column to reports (using a fast heuristic like `cl100k_base` or simple byte mapping).
- **Budget Packing**: `tokmd suggest --budget 128k` — Suggest a list of "high signal" files (based on complexity/size) that fit within a specific token limit.
- **Cost Calculator**: `tokmd cost` — Estimate the API cost to embed/index the current repo state.

### B. "Change Detection" (The Diff Engine)
*Goal: Answer "What changed in the repo shape?"*

- **`tokmd diff --from A.json --to B.json`**: Compare two receipts (low risk, no git plumbing).
- **Git Integration**: `tokmd diff --git HEAD~1`.

### C. Persistent Configuration (`tokmd.toml`)
*Goal: "Set and forget" repo-specific settings.*

- Define `module_roots`, `redact` defaults, and `ignore` patterns in a file.
- Support `[view.llm]` profiles (e.g., `tokmd --view llm`).

### D. Workflow Integration
- **`tokmd run`**: Write canonical receipts (`lang.json`, `module.json`, `export.jsonl`) into a `.runs/` directory.
- **GitHub Action**: Auto-comment on PRs with size delta.
