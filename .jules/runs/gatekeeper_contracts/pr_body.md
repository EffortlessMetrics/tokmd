## 💡 Summary
Added baseline schema tests to prevent version drift. The `BASELINE_VERSION` constant was defined in `crates/tokmd-analysis-types/src/lib.rs`, `docs/SCHEMA.md`, and `docs/baseline.schema.json`, but no automated gate ensured they stayed perfectly synchronized.

## 🎯 Why
Schema or contract definitions can easily drift when versions are manually bumped in some places but forgotten in others. As a Gatekeeper, implementing automated enforcement across these truth sources prevents inconsistencies and ensures deterministic outputs remain sound across code and docs.

## 🔎 Evidence
- `crates/tokmd-analysis-types/src/lib.rs` defines `pub const BASELINE_VERSION: u32 = 1;`
- `docs/SCHEMA.md` documents `BASELINE_VERSION`
- `docs/baseline.schema.json` requires `baseline_version: 1`
- `cargo test -p xtask` passed with the newly added tests guarding these relationships.

## 🧭 Options considered
### Option A (recommended)
- Add verification steps to `xtask/tests/docs_w43.rs` and `xtask/tests/docs_schema_w72.rs` to validate `BASELINE_VERSION` sync between source, documentation, and the JSON schema.
- This perfectly matches the Gatekeeper role by adding missing schema invariants without excessive abstractions.
- Trade-offs: Minor execution overhead in tests, high confidence in version alignment.

### Option B
- Add a dedicated integration test binary specifically for `baseline.schema.json`.
- When to choose it instead: If the JSON schema validation is extremely expensive or requires massive fixtures.
- Trade-offs: Increases integration test scaffolding overhead when it's much simpler to bolt on to the existing robust `xtask` docs testing suite.

## ✅ Decision
Chose Option A to centralize doc and schema assertions inside the existing xtask tests designed for this exact purpose.

## 🧱 Changes made (SRP)
- `xtask/tests/docs_w43.rs`: Added `schema_md_baseline_version_matches_source`.
- `xtask/tests/docs_schema_w72.rs`: Added `schema_md_baseline_version_matches_source` and `baseline_schema_json_version_matches_source`.

## 🧪 Verification receipts
```text
running 28 tests
test changelog_follows_keepachangelog ... ok
test changelog_mentions_latest_version ... ok
test changelog_has_unreleased_section ... ok
test reference_cli_commands_section_exists ... ok
test baseline_schema_json_version_matches_source ... ok
...
test schema_md_baseline_version_matches_source ... ok
...
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

running 15 tests
test baseline_schema_json_is_valid ... ok
test docs_all_md_files_are_nonempty ... ok
...
test schema_md_baseline_version_matches_source ... ok
...
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

## 🧭 Telemetry
- Change shape: Test enhancement
- Blast radius: Restricted strictly to test scaffolding; no runtime logic affected.
- Risk class + why: Low risk. Failing test on drift provides immediate feedback before merge.
- Rollback: Revert the added test functions.
- Gates run: `cargo test -p xtask` (verified sync logic passes on valid inputs).

## 🗂️ .jules artifacts
- `.jules/runs/gatekeeper_contracts/envelope.json`
- `.jules/runs/gatekeeper_contracts/decision.md`
- `.jules/runs/gatekeeper_contracts/receipts.jsonl`
- `.jules/runs/gatekeeper_contracts/result.json`
- `.jules/runs/gatekeeper_contracts/pr_body.md`

## 🔜 Follow-ups
None identified.
