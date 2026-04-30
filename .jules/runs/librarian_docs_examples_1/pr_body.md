## 💡 Summary
Added a missing test to `xtask/tests/docs_w43.rs` to ensure `CONTEXT_BUNDLE_SCHEMA_VERSION` in `crates/tokmd-types/src/lib.rs` matches `docs/SCHEMA.md`. This closes a test coverage gap where all other schema versions were verified for docs drift except this one.

## 🎯 Why
`xtask/tests/docs_w43.rs` checks for schema documentation sync. It tests `SCHEMA_VERSION`, `ANALYSIS_SCHEMA_VERSION`, `COCKPIT_SCHEMA_VERSION`, `CONTEXT_SCHEMA_VERSION`, and `HANDOFF_SCHEMA_VERSION`, but it completely missed `CONTEXT_BUNDLE_SCHEMA_VERSION` which is also documented in `docs/SCHEMA.md`. This omission could lead to silent documentation drift if the context bundle schema version is updated.

## 🔎 Evidence
- Path: `xtask/tests/docs_w43.rs`
- Finding: Missing test `schema_md_context_bundle_version_matches_source`
- Receipt: `cargo test -p xtask` passes after adding the test.

## 🧭 Options considered
### Option A (recommended)
- Add a test `schema_md_context_bundle_version_matches_source` to `xtask/tests/docs_w43.rs`.
- Why it fits: It closes a gap in documentation drift prevention, aligning with the Librarian persona.
- Trade-offs:
  - Structure: Improves test suite structural completeness.
  - Velocity: Negligible test execution cost.
  - Governance: Strengthens the anti-drift gating.

### Option B
- Document the gap as a learning PR.
- When to choose it: If we cannot easily write the test.
- Trade-offs: Leaves a clear gap in documentation verification.

## ✅ Decision
Option A was chosen. Adding this executable test directly serves the Librarian persona`s mission to improve executable coverage to prevent silent doc drift.

## 🧱 Changes made (SRP)
- `xtask/tests/docs_w43.rs`: Added `schema_md_context_bundle_version_matches_source` test.

## 🧪 Verification receipts
```text
$ cargo test -p xtask
...
test schema_md_context_bundle_version_matches_source ... ok
...
test result: ok.
```

## 🧭 Telemetry
- Change shape: Test addition
- Blast radius: Testing (No production code changes)
- Risk class: Low - Test only
- Rollback: Revert the commit touching `xtask/tests/docs_w43.rs`.
- Gates run: cargo test -p xtask, cargo check, cargo fmt, cargo clippy, cargo xtask docs --check

## 🗂️ .jules artifacts
- `.jules/runs/librarian_docs_examples_1/envelope.json`
- `.jules/runs/librarian_docs_examples_1/decision.md`
- `.jules/runs/librarian_docs_examples_1/receipts.jsonl`
- `.jules/runs/librarian_docs_examples_1/result.json`
- `.jules/runs/librarian_docs_examples_1/pr_body.md`

## 🔜 Follow-ups
None.
