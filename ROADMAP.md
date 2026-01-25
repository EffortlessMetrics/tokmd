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
| **v0.9.0** | 🚧 **Next** | Assurance: Integration tests, Golden snapshots, Edge case verification. |
| **v1.0.0** | 📅 Planned | Stability: Frozen schema, Final docs, Official release. |

---

## 🛠️ Remaining Workstreams

### 1. Core Confidence (Testing & Determinism)
*Current State: Unit tests exist for models. No integration tests.*

To reach v1.0, we must guarantee that `tokmd` outputs do not drift unexpectedly. We need a "Golden Test" suite that runs the binary against a static corpus of files and asserts the output matches byte-for-byte.

- **Action**: Add `tests/` directory with `assert_cmd` and `insta` (or file-based diffing).
- **Action**: Create a synthetic test corpus (files with various languages, weird paths, hidden files).
- **Action**: Verify `redact` logic produces identical hashes across platforms (Windows vs Linux paths).

### 2. The Schema Contract
*Current State: Schema exists in code (`schema_version: 1`), but is not formally documented.*

For `tokmd` to be a "receipt", the JSON schema must be treated as a public API.

- **Action**: Extract the JSON schema to a `schemas/tokmd-v1.json` file (or similar) documentation.
- **Action**: Verify that `schema_version` increments if fields change.
- **Action**: Document the `kind` field (Parent vs Child) and `embedded` behavior explicitly.

### 3. Developer Experience (DX)
*Current State: CLI args exist, help text is decent.*

- **Action**: Audit `--help` output for clarity.
- **Action**: Ensure error messages (e.g., file permissions, invalid globs) are actionable and don't panic.
- **Action**: Verify behavior when `tokei` config files are present vs absent (`--config` flag).

---

## 📅 Detailed Milestone Plan

### ✅ Milestone 1–4 (Completed in v0.2.0)
- **Hygiene**: Metadata, cleanups.
- **Schema**: `LangReceipt`, `ModuleReceipt`, `ExportMeta` structs implemented.
- **Semantics**: Unified `--children` flag, consistent module aggregation.
- **Ergonomics**: `--min-code`, `--max-rows`, `--redact`, `--strip-prefix` implemented.

### 🚧 Milestone 5: The Test Harness (Target: v0.9.0)
**Goal**: Refactoring is safe; Output is frozen.

- [x] **Infrastructure**: Add `dev-dependencies` (`assert_cmd`, `predicates`, `tempfile`).
- [x] **Golden Tests**:
    - Run `tokmd export` on a test fixture.
    - Snapshot the JSONL output.
    - Ensure it passes on CI (Windows/Linux/Mac).
- [x] **Path Normalization Verification**:
    - Ensure `\` vs `/` doesn't affect sorting or hashing in receipts.
- [x] **Redaction Verification**:
    - Verify `tokmd export --redact all` leaks no PII in the snapshot.

### 📝 Milestone 6: Documentation & Polish (Target: v0.9.5)
**Goal**: Users understand the "Receipt" concept.

- [ ] **Recipe Book**: Add `docs/recipes.md`:
    - "How to track repo growth over time"
    - "How to feed a codebase to an LLM safely"
    - "How to audit vendor dependencies"
- [ ] **Schema Docs**: Document the fields of `LangReceipt` and `ExportRow`.
- [ ] **Final Argument Audit**: Rename/alias flags if they feel clunky (e.g., check `no_ignore_vcs` usability).

### 🚀 Milestone 7: v1.0.0 Launch
**Goal**: Stability.

- [ ] **SemVer Lock**: Commit to not breaking the JSON schema without v2.0.
- [ ] **Crates.io**: Final publish.
- [ ] **GitHub Release**: binaries attached.

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

- **`tokmd diff <old.json> <new.json>`**: A dedicated command to compare two receipts.
    - Output: "Rust grew by 15%, 4 new modules added."
    - Output: Markdown summary of changes for PR comments.
- **Git Integration**: `tokmd diff --git HEAD~1` (automatically runs scan on old ref).

### C. Persistent Configuration
*Goal: "Set and forget" repo-specific settings.*

- **`.tokmd.toml`**: Support a config file for project-specific settings so users don't need CLI flags every time.
    - `module_roots = ["packages", "services"]`
    - `redact = "paths"`
    - `ignore_hidden = true`

### D. Ecosystem Integration
- **GitHub Action**: `uses: effortlessmetrics/tokmd-action@v1`.
    - Auto-comment on PRs with size delta.
    - Fail build if `redact` is not used in public artifacts.
- **Pre-commit Hook**: Ensure `tokmd export` is up-to-date in the repo.
