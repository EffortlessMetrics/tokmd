## 💡 Summary
Updated `docs/SCHEMA.md`, `docs/design.md`, `docs/architecture.md`, `docs/agent-context/review-invariants.md`, and `docs/specification.md` to properly document the new `TOOL_SCHEMA_VERSION`.

## 🎯 Why
The `TOOL_SCHEMA_VERSION` constant was added in `crates/tokmd/src/tool_schema.rs` and integrated into `xtask bump`, but it was never formally documented in the project's markdown references and invariants alongside other receipt/schema versions, creating a gap in governance and version consistency.

## 🔎 Evidence
- `xtask/src/cli.rs` and `xtask/src/tasks/bump.rs` explicitly parse `TOOL_SCHEMA_VERSION` next to `SCHEMA_VERSION`, `ANALYSIS_SCHEMA_VERSION`, etc.
- `crates/tokmd/src/tool_schema.rs` defines `pub const TOOL_SCHEMA_VERSION: u32 = 1;`.
- The `docs/SCHEMA.md` and related design files do not mention it.

## 🧭 Options considered
### Option A (recommended)
- Add `TOOL_SCHEMA_VERSION` into `docs/SCHEMA.md`'s version history, `docs/design.md`, `docs/architecture.md`, `docs/agent-context/review-invariants.md`, and `docs/specification.md`.
- Maintains a cohesive set of governance and design documentation.
- Trade-offs: Minor documentation churn, but strongly aligns Structure and Governance.

### Option B
- Leave `TOOL_SCHEMA_VERSION` out of the documentation and consider it a purely internal constant.
- Suitable if it was not meant for the public schema registry.
- Trade-offs: Weakens the Schema Version Invariant and leaves `xtask bump` tools out of sync with the documentation.

## ✅ Decision
Option A. `TOOL_SCHEMA_VERSION` acts identically to the other exported schema constants and is expected to follow the Schema Version Invariant.

## 🧱 Changes made (SRP)
- Added `TOOL_SCHEMA_VERSION` tracking to `docs/SCHEMA.md`.
- Added `TOOL_SCHEMA_VERSION` reference to `docs/design.md`.
- Added `TOOL_SCHEMA_VERSION` reference to `docs/architecture.md`.
- Added `TOOL_SCHEMA_VERSION` reference to `docs/agent-context/review-invariants.md`.
- Added `TOOL_SCHEMA_VERSION` reference to `docs/specification.md`.

## 🧪 Verification receipts
```text
$ cargo xtask docs --check
Documentation is up to date.
     Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.31s
     Running `target/debug/xtask docs --check`
```

## 🧭 Telemetry
- Change shape: Docs/Reference Fix
- Blast radius: `docs/` schema, design, and invariants surfaces
- Risk class: Low (documentation only)
- Rollback: Revert the PR
- Gates run: `cargo xtask docs --check`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`

## 🗂️ .jules artifacts
- `.jules/runs/librarian_docs_examples/envelope.json`
- `.jules/runs/librarian_docs_examples/decision.md`
- `.jules/runs/librarian_docs_examples/receipts.jsonl`
- `.jules/runs/librarian_docs_examples/result.json`
- `.jules/runs/librarian_docs_examples/pr_body.md`

## 🔜 Follow-ups
None.
