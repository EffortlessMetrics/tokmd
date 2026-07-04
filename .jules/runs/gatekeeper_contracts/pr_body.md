## 💡 Summary
This PR tightens our contract determinism testing by ensuring that bumps to *all* schema versions (`COCKPIT_SCHEMA_VERSION`, `HANDOFF_SCHEMA_VERSION`, `CONTEXT_SCHEMA_VERSION`, `CONTEXT_BUNDLE_SCHEMA_VERSION`, `TOOL_SCHEMA_VERSION`) are documented in the `CHANGELOG.md`.

## 🎯 Why
Previously, the workspace tests lacked full coverage to ensure all schema version constants were recorded in the changelog upon bumps. By adding explicit changelog document verification tests for cockpit, handoff, context, context bundle, and tool schema versions in `xtask`, this PR guarantees that our release metadata and documentation stay perfectly aligned with code.

## 🔎 Evidence
- File path: `xtask/tests/docs_schema_w72.rs`
- Finding: `docs_schema_w72.rs` tests schema alignment but did not verify `CHANGELOG.md` references for cockpit, handoff, context, context bundle, and tool schema versions.
- Command receipt:
```text
cargo test -p xtask --test docs_schema_w72
```

## 🧭 Options considered
### Option A (recommended)
- Add `changelog_documents_cockpit_schema_version`, `changelog_documents_handoff_schema_version`, etc. tests to `xtask/tests/docs_schema_w72.rs`.
- why it fits this repo and shard: It directly improves our contract determinism testing by enforcing that documentation about schema bumps stays in sync with code bumps, within the allowed `xtask` directory.
- trade-offs: Structure/Governance (better contract/schema alignment testing) over Velocity (slight increase in tests).

### Option B
- Manually verify these without automated tests.
- when to choose it instead: If the documentation is automatically generated.
- trade-offs: Relies on human memory and is prone to drift.

## ✅ Decision
Option A was chosen. Adding explicit schema sync tests for the remaining schema families locks in the contract and ensures automated checks prevent future drift.

## 🧱 Changes made (SRP)
- `xtask/tests/docs_schema_w72.rs`: Added 5 new tests to verify that `CHANGELOG.md` documents bumps for `COCKPIT_SCHEMA_VERSION`, `CONTEXT_SCHEMA_VERSION`, `CONTEXT_BUNDLE_SCHEMA_VERSION`, `HANDOFF_SCHEMA_VERSION`, and `TOOL_SCHEMA_VERSION`.

## 🧪 Verification receipts
```text
{"command": "cargo test -p xtask --test docs_schema_w72"}
```

## 🧭 Telemetry
- Change shape: Test additions
- Blast radius: `xtask/tests/` (Test suite only)
- Risk class: Low (only adds tests)
- Rollback: Revert the commit.
- Gates run: `contracts-determinism` fallback checks

## 🗂️ .jules artifacts
- `.jules/runs/gatekeeper_contracts/envelope.json`
- `.jules/runs/gatekeeper_contracts/decision.md`
- `.jules/runs/gatekeeper_contracts/receipts.jsonl`
- `.jules/runs/gatekeeper_contracts/result.json`
- `.jules/runs/gatekeeper_contracts/pr_body.md`

## 🔜 Follow-ups
None.
